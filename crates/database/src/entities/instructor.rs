use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "instructors")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::instructor_meeting::Entity")]
    InstructorMeetings,
}

impl Related<super::instructor_meeting::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::InstructorMeetings.def()
    }
}

// Many-to-many relationship with meetings
impl Related<super::meeting::Entity> for Entity {
    fn to() -> RelationDef {
        super::instructor_meeting::Relation::Meeting.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::instructor_meeting::Relation::Instructor.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
