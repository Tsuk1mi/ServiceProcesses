use bcrypt::{hash, DEFAULT_COST};
use sea_orm::{ActiveModelTrait, EntityTrait, PaginatorTrait, Set};
use uuid::Uuid;

use crate::domain::errors::DomainError;
use crate::infrastructure::postgres::entity::{app_user, app_user_role};
use crate::infrastructure::postgres::repos::db_err;
use sea_orm::DatabaseConnection;

pub async fn seed_users_if_empty(db: &DatabaseConnection) -> Result<(), DomainError> {
    let n = app_user::Entity::find().count(db).await.map_err(db_err)?;
    if n > 0 {
        return Ok(());
    }

    let admin_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").map_err(|_| DomainError::EmptyField("uuid"))?;
    struct Row {
        id: Uuid,
        subject_id: Uuid,
        username: &'static str,
        password: &'static str,
        roles: &'static [&'static str],
    }

    let rows = [
        Row {
            id: admin_id,
            subject_id: admin_id,
            username: "admin",
            password: "admin",
            roles: &["admin", "dispatcher", "supervisor"],
        },
    ];

    for row in rows {
        let password = row.password.to_string();
        let h = tokio::task::spawn_blocking(move || hash(password, DEFAULT_COST))
            .await
            .map_err(|_| DomainError::EmptyField("bcrypt"))?
            .map_err(|_| DomainError::EmptyField("bcrypt"))?;

        app_user::ActiveModel {
            id: Set(row.id),
            subject_id: Set(row.subject_id),
            username: Set(row.username.to_string()),
            password_hash: Set(h),
        }
        .insert(db)
        .await
        .map_err(db_err)?;

        for role in row.roles {
            app_user_role::ActiveModel {
                user_id: Set(row.id),
                role: Set((*role).to_string()),
            }
            .insert(db)
            .await
            .map_err(db_err)?;
        }
    }

    Ok(())
}
