use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "meetings")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub component_id: Uuid,
    pub days_pattern: String, // e.g. "MWF"
    pub time_start: Option<Time>,
    pub time_end: Option<Time>,
    pub bldg_room: String, // e.g. "GHC 4102"
    pub campus: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::component::Entity",
        from = "Column::ComponentId",
        to = "super::component::Column::Id"
    )]
    Component,
}

impl Related<super::component::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Component.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
