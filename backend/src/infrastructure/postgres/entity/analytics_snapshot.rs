use sea_orm::entity::prelude::*;
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "analytics_snapshot")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub singleton: String,
    #[sea_orm(column_type = "JsonBinary")]
    pub payload: Value,
    pub updated_at_epoch_sec: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
