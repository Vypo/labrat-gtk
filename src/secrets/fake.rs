use super::Error;

#[derive(Debug)]
pub struct Secrets {}

impl super::SecretsExt for Secrets {
    fn new() -> Result<Self, Error> {
        Ok(Self {})
    }

    fn get(&self) -> Result<Option<String>, Error> {
        Ok(None)
    }

    fn set(&self, cookie: &str) -> Result<(), Error> {
        Err(Error::new("secrets not supported"))
    }

    fn clear(&self) -> Result<(), Error> {
        Ok(())
    }
}
