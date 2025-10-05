use std::sync::Arc;

use dashmap::DashMap;
use uuid::Uuid;

use faucet_core::models::{Channel, User};

#[derive(Debug, Clone)]
pub struct SessionData {
    pub user_id: Uuid,
    pub channel: Channel,
    pub handle: String,
    pub domain: Option<String>,
}

#[derive(Clone, Default)]
pub struct SessionManager {
    inner: Arc<DashMap<String, SessionData>>,
}

impl SessionManager {
    pub fn create(&self, user: &User) -> String {
        let token = Uuid::new_v4().to_string();
        let data = SessionData {
            user_id: user.id,
            channel: user.channel.clone(),
            handle: user.handle.clone(),
            domain: user.domain.clone(),
        };
        self.inner.insert(token.clone(), data);
        token
    }

    pub fn get(&self, token: &str) -> Option<SessionData> {
        self.inner.get(token).map(|entry| entry.clone())
    }

    pub fn revoke(&self, token: &str) {
        self.inner.remove(token);
    }
}
