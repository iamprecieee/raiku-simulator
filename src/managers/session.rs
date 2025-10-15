use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::models::session::Session;

#[derive(Clone)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_session(&self) -> Session {
        let session_id = uuid::Uuid::new_v4().to_string();
        let session = Session::new(session_id);

        self.sessions
            .write()
            .await
            .insert(session.id.clone(), session.clone());
        session
    }

    pub async fn get_session(&self, session_id: &str) -> Option<Session> {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.get_mut(session_id) {
            if session.is_expired() {
                sessions.remove(session_id);
                return None;
            }

            session.extend();
            Some(session.clone())
        } else {
            None
        }
    }

    pub async fn validate_session(&self, session_id: &str) -> bool {
        self.get_session(session_id).await.is_some()
    }

    pub async fn cleanup_expired_sessions(&self) -> Vec<String> {
        let mut sessions = self.sessions.write().await;
        let mut removed = Vec::new();

        sessions.retain(|session_id, session| {
            if session.is_expired() {
                removed.push(session_id.clone());
                false
            } else {
                true
            }
        });

        removed
    }

    pub async fn get_session_count(&self) -> usize {
        self.sessions.read().await.len()
    }
}
