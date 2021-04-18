pub(super) mod login {
    use snafu::{Backtrace, Snafu};

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(crate)")]
    pub enum LoginError {
        #[snafu(context(false))]
        InvalidHeaderValue {
            source: reqwest::header::InvalidHeaderValue,
        },
        Client {
            source: labrat::client::ClientError,
            backtrace: Backtrace,
        },
        Exited,
    }
}

pub(super) mod request {
    use snafu::{Backtrace, Snafu};

    use std::convert::Infallible;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(crate)")]
    pub enum RequestError {
        Request {
            source: labrat::client::RequestError<Infallible>,
            backtrace: Backtrace,
        },
        Exited,
    }
}

pub use self::login::LoginError;
pub use self::request::RequestError;
