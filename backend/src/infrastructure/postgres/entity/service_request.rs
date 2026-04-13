use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "service_request")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub asset_id: String,
    #[sea_orm(column_type = "Text")]
    pub description: String,
    pub priority: String,
    pub status: String,
    pub sla_minutes: i32,
    pub created_at_epoch_sec: i64,
    pub owner_user_id: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::asset::Entity",
        from = "Column::AssetId",
        to = "super::asset::Column::Id"
    )]
    Asset,
}

impl Related<super::asset::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Asset.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
