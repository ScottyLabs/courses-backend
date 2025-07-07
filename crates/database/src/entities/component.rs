use models::course_data::ComponentType;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "components")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub course_id: Uuid,
    pub title: String,
    pub component_type: ComponentType,
    pub code: String, // Lecture/section code
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::course::Entity",
        from = "Column::CourseId",
        to = "super::course::Column::Id"
    )]
    Course,
    #[sea_orm(has_many = "super::meeting::Entity")]
    Meetings,
    #[sea_orm(has_many = "super::component_reservation::Entity")]
    ComponentReservations,
}

impl Related<super::course::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Course.def()
    }
}

impl Related<super::meeting::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Meetings.def()
    }
}

impl Related<super::component_reservation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ComponentReservations.def()
    }
}

// Many-to-many relationship with reservations
impl Related<super::reservation::Entity> for Entity {
    fn to() -> RelationDef {
        super::component_reservation::Relation::Reservation.def()
    }

    fn via() -> Option<RelationDef> {
        Some(
            super::component_reservation::Relation::Component
                .def()
                .rev(),
        )
    }
}

impl ActiveModelBehavior for ActiveModel {}
