use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock, RwLockReadGuard};

use uuid::Uuid;

type SessionMap = HashMap<Box<str>, serde_json::Value>;

#[derive(Debug, Clone)]
pub struct Session {
    id: Box<str>,
    pub map: Option<SessionMap>,
}

impl Session {
    pub fn id(&self) -> &Box<str> {
        return &self.id;
    }
}

static SESSIONS: LazyLock<Arc<RwLock<HashMap<Box<str>, SessionMap>>>> = LazyLock::new(|| {
    Arc::new(RwLock::new(HashMap::new()))
});

pub const SESSION_COOKIE: &'static str = "_SESSION_ID";

pub struct Sessions;

impl Sessions {
    pub fn all<'a>() -> RwLockReadGuard<'a, HashMap<Box<str>, SessionMap>> {
        return SESSIONS.read().unwrap();
    }

    pub fn put(session_id: Box<str>, value: SessionMap) -> () {
        let mut sessions = SESSIONS.write().unwrap();
        match sessions.get_mut(&session_id) {
            Some(hashmap) => *hashmap = value,
            None => { sessions.insert(session_id, value); },
        };
    }

    pub fn store(session_id: &str, key: &str, value: serde_json::Value) -> () {
        let mut sessions = SESSIONS.write().unwrap();

        if !sessions.contains_key(session_id) {
            sessions.insert(session_id.to_string().into_boxed_str(), HashMap::new());
        }

        let session = sessions.get_mut(session_id).unwrap();

        session.insert(key.to_string().into_boxed_str(), value);
    }

    pub fn forward(session: Session) -> () {
        return Self::put(session.id, session.map.unwrap_or(HashMap::new()));
    }

    pub fn get(session_id: &str) -> Session {
        let mut sessions = SESSIONS.write().unwrap();
        let session = sessions.remove(session_id);
        sessions.insert(session_id.into(), HashMap::new());

        return Session {
            id: session_id.to_string().into_boxed_str(),
            map: session,
        };
    }

    pub fn store_new_session(value: SessionMap) -> String {
        let session_id = Uuid::new_v4().to_string();
        let mut sessions = SESSIONS.write().unwrap();
        sessions.insert(session_id.clone().into_boxed_str(), value);

        return session_id;
    }

    pub fn new_session() -> String {
        return Self::store_new_session(HashMap::new());
    }

    pub fn clean(session_id: &str) -> () {
        let _ = SESSIONS.write().unwrap().remove(session_id);
    }
}