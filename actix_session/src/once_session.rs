use std::fmt::Debug;

use actix_session::{Session, SessionInsertError};
use actix_web::{dev::ServiceRequest, FromRequest, HttpMessage, HttpRequest};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Default, Clone, Serialize)]
pub struct OnceSession {
    pub flash: Option<String>,
    pub errors: Option<String>,
    pub prev_req: String
}

#[derive(Serialize)]
pub struct OnceSessionMapped<F, E> {
    pub flash: Option<F>,
    pub errors: Option<E>
}

impl OnceSession {
    pub fn map<F, E>(&self) -> Result<OnceSessionMapped<F, E>, anyhow::Error>
    where F: DeserializeOwned + Debug, E: DeserializeOwned + Debug
    {
        let flash = self.flash
            .as_deref()
            .map(serde_json::from_str::<F>)
            .map(|v| {
                v.map_err(|err | anyhow::Error::from(err))
            });

        let flash = match flash {
            None => None,
            Some(parse_result) => {
                if parse_result.is_err() {
                    return Err(parse_result.unwrap_err());
                }

                Some(parse_result.unwrap())
            },
        };

        let errors = self.errors
            .as_deref()
            .map(serde_json::from_str::<E>)
            .map(|v| {
                v.map_err(|err | anyhow::Error::from(err))
            });

        let errors = match errors {
            None => None,
            Some(parse_result) => {
                if parse_result.is_err() {
                    return Err(parse_result.unwrap_err());
                }

                Some(parse_result.unwrap())
            },
        };

        Ok(OnceSessionMapped {
            flash,
            errors
        })
    }
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
pub trait OnceSessionExt {
    fn insert_flash<T>(&self, content: T) -> Result<(), SessionInsertError> where T : Serialize;
    fn insert_errors<T>(&self, errors: T) -> Result<(), SessionInsertError> where T : Serialize;

    fn forward_once_session<F, E>(
        &self,
        once_session: OnceSession
    ) -> Result<(), SessionInsertError> where
        F: DeserializeOwned + Serialize,
        E: DeserializeOwned + Serialize;

    fn flush_flash(&self) -> OnceSession;
    fn current_url(&self, url: &ServiceRequest) -> Result<(), SessionInsertError>;
}

const FLASH_KEY: &'static str = "_flash";
const ERRORS_KEY: &'static str = "_errors";
const PREV_REQ_KEY: &'static str = "_prev_req_url";
const CURR_REQ_KEY: &'static str = "_curr_req_url";

impl OnceSessionExt for Session {
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

    fn current_url(&self, req: &ServiceRequest) -> Result<(), SessionInsertError> {
        let prev_url = self
            .get::<String>(CURR_REQ_KEY)
            .unwrap_or(None)
            .unwrap_or("/".to_string());

        self.insert(PREV_REQ_KEY, prev_url)?;
        self.insert(CURR_REQ_KEY, req.uri().to_string())?;
        return Ok(());
    }

    fn flush_flash(&self) -> OnceSession {
        let flash = self.remove(FLASH_KEY);
        let errors = self.remove(ERRORS_KEY);
        let prev_req: String = self
            .remove(PREV_REQ_KEY)
            .as_deref()
            .map(serde_json::from_str)
            .map_or(None, |v| v.unwrap())
            .unwrap_or("/".into());

        return OnceSession {
            flash,
            errors,
            prev_req,
        };
    }
}
