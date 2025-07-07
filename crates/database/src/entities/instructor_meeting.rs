use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Junction table for many-to-many relationship between instructors and meetings
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "instructor_meetings")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub instructor_id: Uuid,
    pub meeting_id: Uuid,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::instructor::Entity",
        from = "Column::InstructorId",
        to = "super::instructor::Column::Id"
    )]
    Instructor,
    #[sea_orm(
        belongs_to = "super::meeting::Entity",
        from = "Column::MeetingId",
        to = "super::meeting::Column::Id"
    )]
    Meeting,
}

impl Related<super::instructor::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Instructor.def()
    }
}

impl Related<super::meeting::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Meeting.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
