use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl Session {
    pub fn new(id: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            created_at: now,
            last_active: now,
            expires_at: now + Duration::hours(24),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn touch(&mut self) {
        self.last_active = Utc::now();
        self.expires_at = Utc::now() + Duration::hours(24);
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }
}

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
        
        self.sessions.write().await.insert(session.id.clone(), session.clone());
        session
    }

    pub async fn get_session(&self, session_id: &str) -> Option<Session> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            if session.is_expired() {
                sessions.remove(session_id);
                return None;
            }
            
            session.touch();
            Some(session.clone())
        } else {
            None
        }
    }

    pub async fn validate_session(&self, session_id: &str) -> bool {
        self.get_session(session_id).await.is_some()
    }

    pub async fn cleanup_expired_sessions(&self) {
        let mut sessions = self.sessions.write().await;
        sessions.retain(|_, session| !session.is_expired());
    }

    pub async fn get_session_count(&self) -> usize {
        self.sessions.read().await.len()
    }
}