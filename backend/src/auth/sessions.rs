use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use sha2::{Digest, Sha256};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::errors::DomainError;

#[derive(Debug, Clone)]
pub struct AuthSession {
    pub session_id: Uuid,
    pub user_subject_id: Uuid,
    pub client_kind: String,
    pub created_at_epoch_sec: i64,
    pub expires_at_epoch_sec: i64,
    pub revoked_at_epoch_sec: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct IssuedSession {
    pub session: AuthSession,
    pub refresh_token: String,
}

#[async_trait]
pub trait RefreshSessionStore: Send + Sync {
    async fn issue_session(
        &self,
        subject_id: Uuid,
        client: &str,
        refresh_ttl_seconds: i64,
    ) -> Result<IssuedSession, DomainError>;

    async fn refresh_session(
        &self,
        refresh_token: &str,
        refresh_ttl_seconds: i64,
    ) -> Result<IssuedSession, DomainError>;

    async fn revoke_session(&self, session_id: Uuid) -> Result<(), DomainError>;
}

#[derive(Clone, Default)]
pub struct InMemoryRefreshSessionStore {
    sessions: Arc<RwLock<HashMap<Uuid, SessionEntry>>>,
}

#[derive(Debug, Clone)]
struct SessionEntry {
    session: AuthSession,
    refresh_token_hash: String,
}

impl InMemoryRefreshSessionStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl RefreshSessionStore for InMemoryRefreshSessionStore {
    async fn issue_session(
        &self,
        subject_id: Uuid,
        client: &str,
        refresh_ttl_seconds: i64,
    ) -> Result<IssuedSession, DomainError> {
        let now = now_epoch_sec();
        let session_id = Uuid::new_v4();
        let refresh_token = new_refresh_token(session_id);
        let entry = SessionEntry {
            session: AuthSession {
                session_id,
                user_subject_id: subject_id,
                client_kind: client.trim().to_string(),
                created_at_epoch_sec: now,
                expires_at_epoch_sec: now + refresh_ttl_seconds.max(60),
                revoked_at_epoch_sec: None,
            },
            refresh_token_hash: hash_refresh_token(&refresh_token),
        };

        self.sessions.write().await.insert(session_id, entry.clone());

        Ok(IssuedSession {
            session: entry.session,
            refresh_token,
        })
    }

    async fn refresh_session(
        &self,
        refresh_token: &str,
        refresh_ttl_seconds: i64,
    ) -> Result<IssuedSession, DomainError> {
        let session_id = parse_refresh_token_session_id(refresh_token)?;
        let now = now_epoch_sec();
        let mut sessions = self.sessions.write().await;
        let entry = sessions
            .get_mut(&session_id)
            .ok_or(DomainError::Unauthorized("invalid refresh token"))?;

        if entry.session.revoked_at_epoch_sec.is_some() {
            return Err(DomainError::Unauthorized("session revoked"));
        }
        if entry.session.expires_at_epoch_sec <= now {
            return Err(DomainError::Unauthorized("refresh token expired"));
        }
        if entry.refresh_token_hash != hash_refresh_token(refresh_token) {
            return Err(DomainError::Unauthorized("invalid refresh token"));
        }

        let next_refresh_token = new_refresh_token(session_id);
        entry.refresh_token_hash = hash_refresh_token(&next_refresh_token);
        entry.session.expires_at_epoch_sec = now + refresh_ttl_seconds.max(60);
        entry.session.revoked_at_epoch_sec = None;

        Ok(IssuedSession {
            session: entry.session.clone(),
            refresh_token: next_refresh_token,
        })
    }

    async fn revoke_session(&self, session_id: Uuid) -> Result<(), DomainError> {
        if let Some(entry) = self.sessions.write().await.get_mut(&session_id) {
            entry.session.revoked_at_epoch_sec = Some(now_epoch_sec());
        }
        Ok(())
    }
}

pub(crate) fn hash_refresh_token(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let digest = hasher.finalize();
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

pub(crate) fn new_refresh_token(session_id: Uuid) -> String {
    let secret = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    format!("{session_id}.{secret}")
}

pub(crate) fn parse_refresh_token_session_id(refresh_token: &str) -> Result<Uuid, DomainError> {
    let raw_session_id = refresh_token
        .split('.')
        .next()
        .ok_or(DomainError::Unauthorized("invalid refresh token"))?;
    Uuid::parse_str(raw_session_id).map_err(|_| DomainError::Unauthorized("invalid refresh token"))
}

pub(crate) fn now_epoch_sec() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
