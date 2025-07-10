use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Index on courses for common query patterns
        manager
            .create_index(
                Index::create()
                    .name("idx_courses_season_year")
                    .table(Courses::Table)
                    .col(Courses::Season)
                    .col(Courses::Year)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_courses_number")
                    .table(Courses::Table)
                    .col(Courses::Number)
                    .to_owned(),
            )
            .await?;

        // Index on components.course_id for faster joins
        manager
            .create_index(
                Index::create()
                    .name("idx_components_course_id")
                    .table(Components::Table)
                    .col(Components::CourseId)
                    .to_owned(),
            )
            .await?;

        // Index on meetings.component_id for faster joins
        manager
            .create_index(
                Index::create()
                    .name("idx_meetings_component_id")
                    .table(Meetings::Table)
                    .col(Meetings::ComponentId)
                    .to_owned(),
            )
            .await?;

        // Indexes on instructor_meetings for faster many-to-many lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_instructor_meetings_instructor_id")
                    .table(InstructorMeetings::Table)
                    .col(InstructorMeetings::InstructorId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_instructor_meetings_meeting_id")
                    .table(InstructorMeetings::Table)
                    .col(InstructorMeetings::MeetingId)
                    .to_owned(),
            )
            .await?;

        // Indexes on component_reservations for faster many-to-many lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_component_reservations_component_id")
                    .table(ComponentReservations::Table)
                    .col(ComponentReservations::ComponentId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_component_reservations_reservation_id")
                    .table(ComponentReservations::Table)
                    .col(ComponentReservations::ReservationId)
                    .to_owned(),
            )
            .await?;

        // Index on instructors.name for faster lookups during data insertion
        manager
            .create_index(
                Index::create()
                    .name("idx_instructors_name")
                    .table(Instructors::Table)
                    .col(Instructors::Name)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop indexes in reverse order
        manager
            .drop_index(Index::drop().name("idx_instructors_name").to_owned())
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_component_reservations_reservation_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_component_reservations_component_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_instructor_meetings_meeting_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_instructor_meetings_instructor_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(Index::drop().name("idx_meetings_component_id").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_components_course_id").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_courses_number").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_courses_season_year").to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Courses {
    Table,
    Season,
    Year,
    Number,
}

#[derive(Iden)]
enum Components {
    Table,
    CourseId,
}

#[derive(Iden)]
enum Meetings {
    Table,
    ComponentId,
}

#[derive(Iden)]
enum InstructorMeetings {
    Table,
    InstructorId,
    MeetingId,
}

#[derive(Iden)]
enum ComponentReservations {
    Table,
    ComponentId,
    ReservationId,
}

#[derive(Iden)]
enum Instructors {
    Table,
    Name,
}
