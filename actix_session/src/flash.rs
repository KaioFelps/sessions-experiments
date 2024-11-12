use actix_session::{Session, SessionInsertError};
use actix_web::{FromRequest, HttpMessage, HttpRequest};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Default, Clone, Serialize)]
pub struct OnceSession {
    pub flash: Option<String>,
    pub errors: Option<String>,
}

impl FromRequest for OnceSession {
    type Error = actix_web::Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let once_session = req.extensions()
            .get::<OnceSession>()
            .map(|session| session.clone())
            .unwrap_or_default();

        return std::future::ready(Ok(once_session));
    }
}

#[allow(dead_code)]
pub trait Flash {
    fn insert_flash<T>(&self, content: T) -> Result<(), SessionInsertError> where T : Serialize;
    fn insert_errors<T>(&self, errors: T) -> Result<(), SessionInsertError> where T : Serialize;

    fn forward_once_session<F, E>(
        &self,
        once_session: OnceSession
    ) -> Result<(), SessionInsertError> where
        F: DeserializeOwned + Serialize,
        E: DeserializeOwned + Serialize;

    fn flush_flash(&self) -> OnceSession;
}

const FLASH_KEY: &'static str = "flash";
const ERRORS_KEY: &'static str = "errors";

impl Flash for Session {
    fn insert_flash<T>(&self, content: T) -> Result<(), SessionInsertError>
    where T : Serialize
    {
        self.insert(FLASH_KEY, content)?;
        return Ok(());
    }

    fn insert_errors<T>(&self, errors: T) -> Result<(), SessionInsertError>
    where T : Serialize
    {
        self.insert(ERRORS_KEY, errors)?;
        return Ok(());
    }

    /// Deserializes the stringified objects and re-insert them in the Session
    /// (thus, serializing them again).
    fn forward_once_session<F, E>(&self, once_session: OnceSession) -> Result<(), SessionInsertError>
    where F: DeserializeOwned + Serialize, E: DeserializeOwned + Serialize
    {
        self.insert(FLASH_KEY, once_session.flash.map(|f| serde_json::from_str::<F>(&f).unwrap()))?;
        self.insert(ERRORS_KEY, once_session.errors.map(|e| serde_json::from_str::<E>(&e).unwrap()))?;
        return Ok(());
    }

    fn flush_flash(&self) -> OnceSession {
        let flash = self.remove(FLASH_KEY);
        let errors = self.remove(ERRORS_KEY);

        return OnceSession {
            flash,
            errors
        };
    }
}
