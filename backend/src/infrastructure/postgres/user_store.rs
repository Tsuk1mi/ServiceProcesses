use async_trait::async_trait;
use bcrypt::verify;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::domain::errors::DomainError;

use crate::auth::principal::AuthUser;
use crate::auth::UserStore;
use crate::infrastructure::postgres::entity::{app_user, app_user_role};
use crate::infrastructure::postgres::repos::db_err;

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
        let rows = app_user::Entity::find()
            .filter(app_user::Column::Username.eq(username))
            .find_with_related(app_user_role::Entity)
            .all(&self.db)
            .await
            .ok()?;
        let (user, roles) = rows.into_iter().next()?;

        let hash = user.password_hash.clone();
        let pwd = password.to_string();
        let ok = tokio::task::spawn_blocking(move || verify(pwd, &hash).unwrap_or(false))
            .await
            .unwrap_or(false);
        if !ok {
            return None;
        }

        Some(AuthUser {
            sub: user.subject_id,
            roles: roles.into_iter().map(|r| r.role).collect(),
        })
    }

    async fn add_role_for_subject(&self, subject_id: Uuid, role: &str) -> Result<(), DomainError> {
        let user = app_user::Entity::find()
            .filter(app_user::Column::SubjectId.eq(subject_id))
            .one(&self.db)
            .await
            .map_err(db_err)?
            .ok_or(DomainError::NotFound("user"))?;
        let am = app_user_role::ActiveModel {
            user_id: Set(user.id),
            role: Set(role.to_string()),
        };
        app_user_role::Entity::insert(am)
            .on_conflict(
                OnConflict::columns([app_user_role::Column::UserId, app_user_role::Column::Role])
                    .do_nothing()
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(db_err)?;
        Ok(())
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
