use models::requisite::Expr;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "courses")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub number: String,
    pub units: String,
    pub season: String, // F, S, M, N
    pub year: u16,
    pub related_urls: Vec<String>,
    pub special_permission: bool,
    pub description: Option<String>,
    pub prerequisites: Option<Expr>,
    pub corequisites: Vec<String>,
    pub crosslisted: Vec<String>,
    pub notes: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::component::Entity")]
    Components,
}

impl Related<super::component::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Components.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
