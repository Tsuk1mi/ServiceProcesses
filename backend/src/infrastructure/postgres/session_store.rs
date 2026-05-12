use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use uuid::Uuid;

use crate::auth::sessions::{
    AuthSession, IssuedSession, RefreshSessionStore, hash_refresh_token, new_refresh_token,
    now_epoch_sec, parse_refresh_token_session_id,
};
use crate::domain::errors::DomainError;
use crate::infrastructure::postgres::entity::auth_session;
use crate::infrastructure::postgres::repos::db_err;

#[derive(Clone)]
pub struct PgRefreshSessionStore {
    db: DatabaseConnection,
}

impl PgRefreshSessionStore {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl RefreshSessionStore for PgRefreshSessionStore {
    async fn issue_session(
        &self,
        subject_id: Uuid,
        client: &str,
        refresh_ttl_seconds: i64,
    ) -> Result<IssuedSession, DomainError> {
        let now = now_epoch_sec();
        let session_id = Uuid::new_v4();
        let refresh_token = new_refresh_token(session_id);
        let session = AuthSession {
            session_id,
            user_subject_id: subject_id,
            client_kind: client.trim().to_string(),
            created_at_epoch_sec: now,
            expires_at_epoch_sec: now + refresh_ttl_seconds.max(60),
            revoked_at_epoch_sec: None,
        };

        auth_session::ActiveModel {
            session_id: Set(session.session_id),
            user_subject_id: Set(session.user_subject_id),
            refresh_token_hash: Set(hash_refresh_token(&refresh_token)),
            client_kind: Set(session.client_kind.clone()),
            created_at_epoch_sec: Set(session.created_at_epoch_sec),
            expires_at_epoch_sec: Set(session.expires_at_epoch_sec),
            revoked_at_epoch_sec: Set(session.revoked_at_epoch_sec),
        }
        .insert(&self.db)
        .await
        .map_err(db_err)?;

        Ok(IssuedSession {
            session,
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
        let existing = auth_session::Entity::find_by_id(session_id)
            .one(&self.db)
            .await
            .map_err(db_err)?
            .ok_or(DomainError::Unauthorized("invalid refresh token"))?;

        if existing.revoked_at_epoch_sec.is_some() {
            return Err(DomainError::Unauthorized("session revoked"));
        }
        if existing.expires_at_epoch_sec <= now {
            return Err(DomainError::Unauthorized("refresh token expired"));
        }
        if existing.refresh_token_hash != hash_refresh_token(refresh_token) {
            return Err(DomainError::Unauthorized("invalid refresh token"));
        }

        let next_refresh_token = new_refresh_token(existing.session_id);
        let next_expiry = now + refresh_ttl_seconds.max(60);
        let mut active: auth_session::ActiveModel = existing.clone().into();
        active.refresh_token_hash = Set(hash_refresh_token(&next_refresh_token));
        active.expires_at_epoch_sec = Set(next_expiry);
        active.revoked_at_epoch_sec = Set(None);
        active.update(&self.db).await.map_err(db_err)?;

        Ok(IssuedSession {
            session: AuthSession {
                session_id: existing.session_id,
                user_subject_id: existing.user_subject_id,
                client_kind: existing.client_kind,
                created_at_epoch_sec: existing.created_at_epoch_sec,
                expires_at_epoch_sec: next_expiry,
                revoked_at_epoch_sec: None,
            },
            refresh_token: next_refresh_token,
        })
    }

    async fn revoke_session(&self, session_id: Uuid) -> Result<(), DomainError> {
        let Some(existing) = auth_session::Entity::find_by_id(session_id)
            .one(&self.db)
            .await
            .map_err(db_err)?
        else {
            return Ok(());
        };

        let mut active: auth_session::ActiveModel = existing.into();
        active.revoked_at_epoch_sec = Set(Some(now_epoch_sec()));
        active.update(&self.db).await.map_err(db_err)?;
        Ok(())
    }
}
