use models::reservation_type::ReservationType;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "reservations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub student_type: Option<String>,
    pub restriction_type: Option<ReservationType>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
