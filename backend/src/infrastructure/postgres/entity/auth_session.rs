use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "auth_session")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Uuid")]
    pub session_id: Uuid,
    #[sea_orm(column_type = "Uuid")]
    pub user_subject_id: Uuid,
    pub refresh_token_hash: String,
    pub client_kind: String,
    pub created_at_epoch_sec: i64,
    pub expires_at_epoch_sec: i64,
    pub revoked_at_epoch_sec: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
