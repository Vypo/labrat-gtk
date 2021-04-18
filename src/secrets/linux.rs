use secret_service::{Collection, EncryptionType, SecretService};

use std::collections::HashMap;

use super::Error;

pub struct Secrets {
    service: SecretService<'static>,
}

impl Secrets {
    fn collection(&self) -> Result<Collection, Error> {
        let collection = self.service.get_any_collection().map_err(|e| {
            Error::with_source("unable to get any collection", e)
        })?;

        if let Ok(true) = collection.is_locked() {
            collection.unlock().map_err(|e| {
                Error::with_source("unable to unlock collection", e)
            })?;
        }

        Ok(collection)
    }

    fn attributes() -> HashMap<&'static str, &'static str> {
        let mut map = HashMap::with_capacity(1);
        map.insert("labrat", "cookie");
        map
    }
}

impl super::SecretsExt for Secrets {
    fn new() -> Result<Self, Error> {
        let service = SecretService::new(EncryptionType::Dh).map_err(|e| {
            Error::with_source("unable to create secret service", e)
        })?;

        Ok(Self { service })
    }

    fn get(&self) -> Result<Option<String>, Error> {
        let collection = self.collection()?;

        let found =
            collection.search_items(Self::attributes()).map_err(|e| {
                Error::with_source("unable to search collection", e)
            })?;

        let item = match found.first() {
            Some(i) => i,
            None => return Ok(None),
        };

        if let Ok(true) = item.is_locked() {
            item.unlock()
                .map_err(|e| Error::with_source("unable to unlock item", e))?;
        }

        let secret_bytes = item
            .get_secret()
            .map_err(|e| Error::with_source("unable to get secret", e))?;
        let secret = String::from_utf8(secret_bytes)
            .map_err(|e| Error::with_source("unable to decode secret", e))?;

        Ok(Some(secret))
    }

    fn set(&self, cookie: &str) -> Result<(), Error> {
        let collection = self.collection()?;

        collection
            .create_item(
                "cookie",
                Self::attributes(),
                cookie.as_bytes(),
                true,
                "text/plain",
            )
            .map_err(|e| Error::with_source("unable to create item", e))?;

        Ok(())
    }

    fn clear(&self) -> Result<(), Error> {
        let collection = self.collection()?;

        let found =
            collection.search_items(Self::attributes()).map_err(|e| {
                Error::with_source("unable to search collection", e)
            })?;

        if let Some(item) = found.first() {
            item.delete().map_err(|e| {
                Error::with_source("unable to delete secret", e)
            })?;
        }

        Ok(())
    }
}
