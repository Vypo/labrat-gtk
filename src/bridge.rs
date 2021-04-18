pub mod errors;

use futures_channel::oneshot;

use labrat::client::Response;
use labrat::keys::{
    CommentReplyKey, FavKey, JournalKey, SubmissionsKey, ViewKey,
};
use labrat::resources::journal::Journal;
use labrat::resources::msg::others::Others;
use labrat::resources::msg::submissions::Submissions;
use labrat::resources::view::View;

use reqwest::header::HeaderValue;

use self::errors::login::LoginError;
use self::errors::request::RequestError;

use snafu::ResultExt;

use std::convert::Infallible;
use std::thread::{self, JoinHandle};

use tokio::sync::mpsc;

type Error = labrat::client::RequestError<Infallible>;
type ResponseResult<T, E = Error> = std::result::Result<Response<T>, E>;
type ResponseSender<T> = oneshot::Sender<ResponseResult<T>>;

#[derive(Debug)]
enum Message {
    Stop,
    Replace(labrat::client::Client, oneshot::Sender<()>),

    Journal(JournalKey, ResponseSender<Journal>),
    View(ViewKey, ResponseSender<View>),
    Reply(CommentReplyKey, String, oneshot::Sender<Result<(), Error>>),
    Fav(FavKey, ResponseSender<View>),
    Unfav(FavKey, ResponseSender<View>),
    Others(ResponseSender<Others>),
    Submissions(SubmissionsKey, ResponseSender<Submissions>),
    ClearSubmissions(Vec<ViewKey>, oneshot::Sender<Result<(), Error>>),
}

#[derive(Debug, Clone)]
pub struct Client {
    sender: mpsc::UnboundedSender<Message>,
}

impl Client {
    async fn recv<T>(
        &self,
        receiver: oneshot::Receiver<ResponseResult<T>>,
    ) -> Result<T, RequestError> {
        let resp = receiver
            .await
            .map_err(|_| RequestError::Exited)?
            .context(errors::request::Request)?;

        // TODO: Update notification counts and such.

        Ok(resp.page)
    }

    pub fn stop(&self) {
        self.sender.send(Message::Stop).ok();
    }

    pub async fn login(&self, cookies: &str) -> Result<(), LoginError> {
        let value = HeaderValue::from_str(cookies)?;

        let client = labrat::client::Client::with_cookies(value)
            .context(errors::login::Client)?;

        let (reply, recv) = oneshot::channel();

        self.sender
            .send(Message::Replace(client, reply))
            .map_err(|_| LoginError::Exited)?;

        recv.await.map_err(|_| LoginError::Exited)?;

        Ok(())
    }

    pub async fn submissions(
        &self,
        key: SubmissionsKey,
    ) -> Result<Submissions, RequestError> {
        let (reply, recv) = oneshot::channel();

        self.sender
            .send(Message::Submissions(key, reply))
            .map_err(|_| RequestError::Exited)?;

        Ok(self.recv(recv).await?)
    }
}

#[derive(Debug)]
pub struct Bridge {
    thread: JoinHandle<()>,
    client: Client,
}

impl Bridge {
    pub fn spawn() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        let thread = thread::Builder::new()
            .name("labrat-bridge".into())
            .spawn(move || Self::run(receiver))
            .unwrap();

        let client = Client { sender };

        Self { thread, client }
    }

    pub fn join(self) {
        self.client.stop();
        if let Err(e) = self.thread.join() {
            std::panic::resume_unwind(e)
        }
    }

    pub fn client(&self) -> Client {
        self.client.clone()
    }

    #[tokio::main(flavor = "current_thread")]
    async fn run(mut receiver: mpsc::UnboundedReceiver<Message>) {
        eprintln!("labrat running");
        let mut client = labrat::client::Client::new().unwrap();

        while let Some(message) = receiver.recv().await {
            eprintln!("labrat got msg: {:?}", message);
            match message {
                Message::Stop => break,

                Message::Replace(c, reply) => {
                    client = c;
                    reply.send(()).ok();
                }

                Message::Journal(key, reply) => {
                    let result = client.journal(key).await;
                    reply.send(result).ok();
                }

                Message::View(key, reply) => {
                    let result = client.view(key).await;
                    reply.send(result).ok();
                }

                Message::Reply(key, text, reply) => {
                    let result = client.reply(key, &text).await;
                    reply.send(result).ok();
                }

                Message::Fav(key, reply) => {
                    let result = client.fav(key).await;
                    reply.send(result).ok();
                }

                Message::Unfav(key, reply) => {
                    let result = client.unfav(key).await;
                    reply.send(result).ok();
                }

                Message::Others(reply) => {
                    let result = client.others().await;
                    reply.send(result).ok();
                }

                Message::Submissions(key, reply) => {
                    let result = client.submissions(key).await;
                    reply.send(result).ok();
                }

                Message::ClearSubmissions(keys, reply) => {
                    let result = client.clear_submissions(keys).await;
                    reply.send(result).ok();
                }
            }
        }
    }
}
