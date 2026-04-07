use std::collections::HashMap;

use async_trait::async_trait;
use bcrypt::{hash, verify, DEFAULT_COST};
use uuid::Uuid;

use crate::auth::principal::AuthUser;
use crate::domain::errors::DomainError;

#[derive(Clone)]
struct Account {
    id: Uuid,
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
        let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000002")
            .map_err(|_| DomainError::EmptyField("user id"))?;

        by_username.insert(
            "admin".to_string(),
            Account {
                id: admin_id,
                password_hash: hash("admin", DEFAULT_COST).map_err(|_| DomainError::EmptyField("bcrypt"))?,
                roles: vec!["admin".to_string(), "dispatcher".to_string(), "supervisor".to_string()],
            },
        );
        by_username.insert(
            "dispatcher".to_string(),
            Account {
                id: admin_id,
                password_hash: hash("dispatcher", DEFAULT_COST).map_err(|_| DomainError::EmptyField("bcrypt"))?,
                roles: vec!["dispatcher".to_string()],
            },
        );
        by_username.insert(
            "user".to_string(),
            Account {
                id: user_id,
                password_hash: hash("user", DEFAULT_COST).map_err(|_| DomainError::EmptyField("bcrypt"))?,
                roles: vec!["user".to_string()],
            },
        );

        let tech_id = Uuid::parse_str("00000000-0000-0000-0000-000000000003")
            .map_err(|_| DomainError::EmptyField("tech id"))?;
        by_username.insert(
            "technician".to_string(),
            Account {
                id: tech_id,
                password_hash: hash("technician", DEFAULT_COST)
                    .map_err(|_| DomainError::EmptyField("bcrypt"))?,
                roles: vec!["technician".to_string()],
            },
        );

        Ok(Self { by_username })
    }

    pub fn demo_admin_id() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").expect("uuid")
    }

    pub fn demo_user_id() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000002").expect("uuid")
    }

    pub fn demo_technician_id() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000003").expect("uuid")
    }
}

#[async_trait]
pub trait UserStore: Send + Sync {
    async fn verify(&self, username: &str, password: &str) -> Option<AuthUser>;
}

#[async_trait]
impl UserStore for InMemoryUserStore {
    async fn verify(&self, username: &str, password: &str) -> Option<AuthUser> {
        let this = self.clone();
        let u = username.to_string();
        let p = password.to_string();
        tokio::task::spawn_blocking(move || {
            let acc = this.by_username.get(&u)?;
            if verify(p, &acc.password_hash).ok()? {
                Some(AuthUser {
                    sub: acc.id,
                    roles: acc.roles.clone(),
                })
            } else {
                None
            }
        })
        .await
        .ok()
        .flatten()
    }
}
