use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock, RwLockReadGuard, RwLockWriteGuard};
use actix_session::storage::{generate_session_key, LoadError, SaveError, SessionKey, SessionStore, UpdateError};
use actix_web::cookie::time::Duration;

pub(crate) type SessionState = HashMap<String, String>;

struct Session {
    session: SessionState,
    ttl: Duration,
} 

static SESSIONS: LazyLock<Arc<RwLock<HashMap<Box<str>, Session>>>> = LazyLock::new(|| {
    Arc::new(RwLock::new(HashMap::new()))
});

fn write_sessions<'a>() -> RwLockWriteGuard<'a, HashMap<Box<str>, Session>> {
    return SESSIONS.write().unwrap_or_else(|mut e| {
        **e.get_mut() = HashMap::new();
        SESSIONS.clear_poison();
        e.into_inner()
    });
}

fn read_sessions<'a>() -> RwLockReadGuard<'a, HashMap<Box<str>, Session>> {
    if SESSIONS.is_poisoned() {
        drop(write_sessions());
    }
    return SESSIONS.read().unwrap();
}

pub struct StatefulSessions;

impl SessionStore for StatefulSessions
{
    async fn load(
        &self,
        session_key: &SessionKey,
    ) -> Result<Option<SessionState>, LoadError> {
        let sessions = read_sessions();
        return Ok(sessions.get(session_key.as_ref()).map(|s| s.session.clone()));
    }

    async fn save(
        &self,
        session_state: SessionState,
        ttl: &actix_web::cookie::time::Duration,
    ) -> Result<SessionKey, SaveError> {
        let session_key = generate_session_key();
        let mut sessions = write_sessions();
        sessions.insert(session_key.as_ref().to_string().into_boxed_str(), Session {
            session: session_state,
            ttl: ttl.clone(),
        });
        return Ok(session_key);
    }

    async fn update(
        &self,
        session_key: SessionKey,
        session_state: SessionState,
        ttl: &actix_web::cookie::time::Duration,
    ) -> Result<SessionKey, UpdateError> {
        let mut sessions = write_sessions();
        if !sessions.contains_key(session_key.as_ref()) {
            sessions.insert(
                session_key.as_ref().to_string().into_boxed_str(),
                Session { session: session_state, ttl: ttl.clone() }
            );
            return Ok(session_key);
        }

        *sessions.get_mut(session_key.as_ref()).unwrap() = Session {
            session: session_state,
            ttl: ttl.clone()
        };

        return Ok(session_key);
    }

    async fn update_ttl(
        &self,
        session_key: &SessionKey,
        ttl: &actix_web::cookie::time::Duration,
    ) -> Result<(), anyhow::Error> {
        let mut sessions = write_sessions();
        match sessions.get_mut(session_key.as_ref()) {
            None => return Err(anyhow::Error::msg("Session does not exist.")),
            Some(session) => { (*session).ttl = ttl.clone(); },
        };
        
        return Ok(());
    }

    async fn delete(&self, session_key: &SessionKey) -> Result<(), anyhow::Error> {
        let mut sessions = write_sessions();
        if sessions.contains_key(session_key.as_ref()) {
            sessions.remove(session_key.as_ref());
        }

        return Ok(());
    }
}
