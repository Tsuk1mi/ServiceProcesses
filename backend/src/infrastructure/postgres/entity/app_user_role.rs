use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "app_user_role")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Uuid")]
    pub user_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub role: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::app_user::Entity",
        from = "Column::UserId",
        to = "super::app_user::Column::Id"
    )]
    AppUser,
}

impl Related<super::app_user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AppUser.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
