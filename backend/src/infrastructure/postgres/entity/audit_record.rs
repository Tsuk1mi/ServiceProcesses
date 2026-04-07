use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "audit_record")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub request_id: Option<String>,
    pub entity: String,
    pub action: String,
    pub actor_role: String,
    pub actor_id: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub details: String,
    pub created_at_utc: String,
    pub owner_user_id: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
