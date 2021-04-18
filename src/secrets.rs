#[cfg(target_os = "linux")]
mod linux;

#[cfg(not(target_os = "linux"))]
mod fake;

#[cfg(target_os = "linux")]
pub use self::linux::*;

#[cfg(not(target_os = "linux"))]
pub use self::fake::*;

use std::borrow::Cow;
use std::fmt;

#[derive(Debug)]
pub struct Error {
    message: Cow<'static, str>,
    source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl Error {
    #[allow(unused)]
    fn new<S: Into<Cow<'static, str>>>(message: S) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    #[allow(unused)]
    fn with_source<S, E>(message: S, source: E) -> Self
    where
        S: Into<Cow<'static, str>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        Self {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)?;

        if let Some(ref source) = self.source {
            write!(f, " (caused by: {})", source)?;
        }

        Ok(())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(Box::as_ref)
            .map(|x| x as &(dyn std::error::Error + 'static))
    }
}

pub trait SecretsExt: Sized {
    fn new() -> Result<Self, Error>;

    fn set(&self, cookie: &str) -> Result<(), Error>;
    fn get(&self) -> Result<Option<String>, Error>;
    fn clear(&self) -> Result<(), Error>;
}
