use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "app_user")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Uuid")]
    pub id: Uuid,
    #[sea_orm(column_type = "Uuid")]
    pub subject_id: Uuid,
    #[sea_orm(unique)]
    pub username: String,
    pub password_hash: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::app_user_role::Entity")]
    AppUserRole,
}

impl Related<super::app_user_role::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AppUserRole.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
