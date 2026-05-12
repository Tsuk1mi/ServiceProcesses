use std::collections::HashMap;

use async_trait::async_trait;
use bcrypt::{hash, verify, DEFAULT_COST};
use uuid::Uuid;

use crate::auth::principal::AuthUser;
use crate::domain::errors::DomainError;

#[derive(Debug, Clone)]
pub struct AuthIdentity {
    pub auth: AuthUser,
    pub username: String,
}

#[derive(Clone)]
struct Account {
    id: Uuid,
    username: String,
    password_hash: String,
    roles: Vec<String>,
}

#[derive(Clone)]
pub struct InMemoryUserStore {
    by_username: HashMap<String, Account>,
}

impl InMemoryUserStore {
    pub fn demo() -> Result<Self, DomainError> {
        let mut by_username = HashMap::new();

        let admin_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
            .map_err(|_| DomainError::EmptyField("admin id"))?;

        by_username.insert(
            "admin".to_string(),
            Account {
                id: admin_id,
                username: "admin".to_string(),
                password_hash: hash("admin", DEFAULT_COST).map_err(|_| DomainError::EmptyField("bcrypt"))?,
                roles: vec!["admin".to_string(), "dispatcher".to_string(), "supervisor".to_string()],
            },
        );

        Ok(Self { by_username })
    }

    pub fn demo_admin_id() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").expect("uuid")
    }
}

#[async_trait]
pub trait UserStore: Send + Sync {
    async fn verify(&self, username: &str, password: &str) -> Option<AuthIdentity>;

    async fn find_by_subject(&self, subject_id: Uuid) -> Option<AuthIdentity>;

    /// Назначить роль по `subject_id` (UUID субъекта в JWT). Реализовано для PostgreSQL.
    async fn add_role_for_subject(&self, _subject_id: Uuid, _role: &str) -> Result<(), DomainError> {
        Err(DomainError::EmptyField("add_role_for_subject not supported"))
    }
}

#[async_trait]
impl UserStore for InMemoryUserStore {
    async fn verify(&self, username: &str, password: &str) -> Option<AuthIdentity> {
        let this = self.clone();
        let u = username.to_string();
        let p = password.to_string();
        tokio::task::spawn_blocking(move || {
            let acc = this.by_username.get(&u)?;
            if verify(p, &acc.password_hash).ok()? {
                Some(AuthIdentity {
                    auth: AuthUser {
                        sub: acc.id,
                        roles: acc.roles.clone(),
                        session_id: None,
                    },
                    username: acc.username.clone(),
                })
            } else {
                None
            }
        })
        .await
        .ok()
        .flatten()
    }

    async fn find_by_subject(&self, subject_id: Uuid) -> Option<AuthIdentity> {
        self.by_username
            .values()
            .find(|account| account.id == subject_id)
            .map(|account| AuthIdentity {
                auth: AuthUser {
                    sub: account.id,
                    roles: account.roles.clone(),
                    session_id: None,
                },
                username: account.username.clone(),
            })
    }

    async fn add_role_for_subject(&self, _subject_id: Uuid, _role: &str) -> Result<(), DomainError> {
        Err(DomainError::Forbidden("in-memory store: use PostgreSQL for role management"))
    }
}
