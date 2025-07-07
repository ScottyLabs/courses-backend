use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Junction table for many-to-many relationship between components and reservations
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "component_reservations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub component_id: Uuid,
    pub reservation_id: Uuid,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::component::Entity",
        from = "Column::ComponentId",
        to = "super::component::Column::Id"
    )]
    Component,
    #[sea_orm(
        belongs_to = "super::reservation::Entity",
        from = "Column::ReservationId",
        to = "super::reservation::Column::Id"
    )]
    Reservation,
}

impl Related<super::component::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Component.def()
    }
}

impl Related<super::reservation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Reservation.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
