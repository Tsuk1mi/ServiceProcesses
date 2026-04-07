use async_trait::async_trait;
use bcrypt::verify;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::auth::principal::AuthUser;
use crate::auth::UserStore;
use crate::infrastructure::postgres::entity::{app_user, app_user_role};

#[derive(Clone)]
pub struct PgUserStore {
    db: DatabaseConnection,
}

impl PgUserStore {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserStore for PgUserStore {
    async fn verify(&self, username: &str, password: &str) -> Option<AuthUser> {
        let user = app_user::Entity::find()
            .filter(app_user::Column::Username.eq(username))
            .one(&self.db)
            .await
            .ok()??;

        let hash = user.password_hash.clone();
        let pwd = password.to_string();
        let ok = tokio::task::spawn_blocking(move || verify(pwd, &hash).unwrap_or(false))
            .await
            .unwrap_or(false);
        if !ok {
            return None;
        }

        let roles = app_user_role::Entity::find()
            .filter(app_user_role::Column::UserId.eq(user.id))
            .all(&self.db)
            .await
            .ok()?;

        Some(AuthUser {
            sub: user.subject_id,
            roles: roles.into_iter().map(|r| r.role).collect(),
        })
    }
}

impl PgUserStore {
    pub fn demo_admin_subject() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").expect("uuid")
    }

    pub fn demo_technician_subject() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000003").expect("uuid")
    }
}
