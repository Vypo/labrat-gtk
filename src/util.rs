use crate::bridge::Client;

use directories::ProjectDirs;

pub use self::error::Error;

use snafu::{Backtrace, OptionExt, ResultExt, Snafu};

use soup::{CacheExt, RequestExt, SessionExt};

use std::rc::Rc;

mod error {
    use super::*;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub enum Error {
        /// Unable to find the standard directories for project files.
        DirectoriesNotFound { backtrace: Backtrace },

        /// Unable to start the glib thread pool.
        Threads {
            backtrace: Backtrace,
            source: glib::Error,
        },

        /// Converting path to string failed.
        PathInvalidUtf8 { backtrace: Backtrace },
    }
}

#[derive(Debug)]
struct Inner {
    dirs: ProjectDirs,
    threads: glib::ThreadPool,
    http: soup::Session,
    http_cache: soup::Cache,
    client: Client,
}

impl Drop for Inner {
    fn drop(&mut self) {
        self.http_cache.dump();
        self.http_cache.flush();
    }
}

// TODO: Util classes are evil, refactor this somehow.
#[derive(Debug, Clone)]
pub struct Util(Rc<Inner>);

impl Util {
    pub fn new(client: Client) -> Result<Self, Error> {
        let dirs = ProjectDirs::from(
            crate::QUALIFIER,
            crate::ORGANIZATION,
            env!("CARGO_PKG_NAME"),
        )
        .context(error::DirectoriesNotFound)?;

        let threads =
            glib::ThreadPool::new_shared(None).context(error::Threads)?;

        // TODO: Set a max size for the HTTP cache?
        let http_cache_dir = dirs.cache_dir().join("soup");
        let http_cache = soup::Cache::new(
            Some(http_cache_dir.to_str().context(error::PathInvalidUtf8)?),
            soup::CacheType::SingleUser,
        );
        http_cache.load();

        let http = soup::Session::new();
        http.add_feature(&http_cache);

        Ok(Util(Rc::new(Inner {
            dirs,
            http,
            http_cache,
            threads,
            client,
        })))
    }

    pub fn client(&self) -> &Client {
        &self.0.client
    }

    pub async fn spawn_background<F, T>(&self, func: F) -> T
    where
        T: 'static + Send,
        F: 'static + Send + FnOnce() -> T,
    {
        // TODO: show an error instead of unwrapping I guess.
        self.0.threads.push_future(func).unwrap().await
    }

    pub fn spawn_local<F, E>(&self, fut: F)
    where
        F: 'static + std::future::Future<Output = Result<(), E>>,
        E: std::error::Error,
    {
        eprintln!("Spawn local?");
        glib::MainContext::default().spawn_local(async move {
            eprintln!("local spawned");
            if let Err(e) = fut.await {
                eprintln!("{}", e);
                std::process::abort();
                // TODO: Show an error instead of exiting I guess.
            }
        });
    }

    pub async fn http_get(
        &self,
        uri: &str,
    ) -> Result<gio::InputStream, glib::Error> {
        let stream = self
            .0
            .http
            .request_http("GET", uri)?
            .send_async_future()
            .await?;
        Ok(stream)
    }

    pub async fn fetch_pixbuf(
        &self,
        uri: &str,
    ) -> Result<gdk_pixbuf::Pixbuf, glib::Error> {
        let stream = self.http_get(uri).await?;
        let pixbuf =
            gdk_pixbuf::Pixbuf::from_stream_async_future(&stream).await?;
        Ok(pixbuf)
    }
}
