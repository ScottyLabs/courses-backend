use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create courses table
        manager
            .create_table(
                Table::create()
                    .table(Courses::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Courses::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Courses::Number).string().not_null())
                    .col(ColumnDef::new(Courses::Units).string().not_null())
                    .col(ColumnDef::new(Courses::Season).string().not_null())
                    .col(ColumnDef::new(Courses::Year).small_unsigned().not_null())
                    .col(ColumnDef::new(Courses::RelatedUrls).json().not_null())
                    .col(
                        ColumnDef::new(Courses::SpecialPermission)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(Courses::Description).text())
                    .col(ColumnDef::new(Courses::Prerequisites).text())
                    .col(ColumnDef::new(Courses::Corequisites).json().not_null())
                    .col(ColumnDef::new(Courses::Crosslisted).json().not_null())
                    .col(ColumnDef::new(Courses::Notes).text())
                    .to_owned(),
            )
            .await?;

        // Create reservations table
        manager
            .create_table(
                Table::create()
                    .table(Reservations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Reservations::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Reservations::StudentType).string())
                    .col(ColumnDef::new(Reservations::RestrictionType).text())
                    .to_owned(),
            )
            .await?;

        // Create instructors table
        manager
            .create_table(
                Table::create()
                    .table(Instructors::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Instructors::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Instructors::Name).string().not_null())
                    .to_owned(),
            )
            .await?;

        // Create components table
        manager
            .create_table(
                Table::create()
                    .table(Components::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Components::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Components::CourseId).uuid().not_null())
                    .col(ColumnDef::new(Components::Title).string().not_null())
                    .col(ColumnDef::new(Components::ComponentType).text().not_null())
                    .col(ColumnDef::new(Components::Code).string().not_null())
                    .col(ColumnDef::new(Components::SyllabusUrl).string())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-components-course_id")
                            .from(Components::Table, Components::CourseId)
                            .to(Courses::Table, Courses::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create component_reservations junction table (many-to-many)
        manager
            .create_table(
                Table::create()
                    .table(ComponentReservations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ComponentReservations::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ComponentReservations::ComponentId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ComponentReservations::ReservationId)
                            .uuid()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-component_reservations-component_id")
                            .from(
                                ComponentReservations::Table,
                                ComponentReservations::ComponentId,
                            )
                            .to(Components::Table, Components::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-component_reservations-reservation_id")
                            .from(
                                ComponentReservations::Table,
                                ComponentReservations::ReservationId,
                            )
                            .to(Reservations::Table, Reservations::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create meetings table
        manager
            .create_table(
                Table::create()
                    .table(Meetings::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Meetings::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Meetings::ComponentId).uuid().not_null())
                    .col(ColumnDef::new(Meetings::DaysPattern).string().not_null())
                    .col(ColumnDef::new(Meetings::TimeBegin).time())
                    .col(ColumnDef::new(Meetings::TimeEnd).time())
                    .col(ColumnDef::new(Meetings::BldgRoom).string().not_null())
                    .col(ColumnDef::new(Meetings::Campus).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-meetings-component_id")
                            .from(Meetings::Table, Meetings::ComponentId)
                            .to(Components::Table, Components::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create instructor_meetings junction table (many-to-many)
        manager
            .create_table(
                Table::create()
                    .table(InstructorMeetings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(InstructorMeetings::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(InstructorMeetings::InstructorId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InstructorMeetings::MeetingId)
                            .uuid()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-instructor_meetings-instructor_id")
                            .from(InstructorMeetings::Table, InstructorMeetings::InstructorId)
                            .to(Instructors::Table, Instructors::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-instructor_meetings-meeting_id")
                            .from(InstructorMeetings::Table, InstructorMeetings::MeetingId)
                            .to(Meetings::Table, Meetings::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create FCE table
        manager
            .create_table(
                Table::create()
                    .table(Evaluations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Evaluations::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Evaluations::ComponentId).uuid().not_null())
                    .col(ColumnDef::new(Evaluations::InstructorId).uuid().not_null())
                    .col(
                        ColumnDef::new(Evaluations::CourseShortName)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Evaluations::CourseLevel).string().not_null())
                    .col(
                        ColumnDef::new(Evaluations::TotalStudents)
                            .small_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Evaluations::NumResponses)
                            .small_unsigned()
                            .not_null(),
                    )
                    // Evaluation ratings (stored as decimals)
                    .col(ColumnDef::new(Evaluations::HoursPerWeek).decimal())
                    .col(ColumnDef::new(Evaluations::InterestInStudentLearning).decimal())
                    .col(ColumnDef::new(Evaluations::ClearlyExplainRequirements).decimal())
                    .col(ColumnDef::new(Evaluations::ClearLearningObjectives).decimal())
                    .col(ColumnDef::new(Evaluations::InstructorProvidesFeedback).decimal())
                    .col(ColumnDef::new(Evaluations::DemonstrateImportance).decimal())
                    .col(ColumnDef::new(Evaluations::ExplainsSubjectMatter).decimal())
                    .col(ColumnDef::new(Evaluations::ShowRespectForStudents).decimal())
                    .col(ColumnDef::new(Evaluations::OverallTeachingRate).decimal())
                    .col(ColumnDef::new(Evaluations::OverallCourseRate).decimal())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-evaluations-component_id")
                            .from(Evaluations::Table, Evaluations::ComponentId)
                            .to(Components::Table, Components::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-evaluations-instructor_id")
                            .from(Evaluations::Table, Evaluations::InstructorId)
                            .to(Instructors::Table, Instructors::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order due to foreign key constraints
        manager
            .drop_table(Table::drop().table(ComponentReservations::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(InstructorMeetings::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Evaluations::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Meetings::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Components::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Reservations::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Instructors::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Courses::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Courses {
    Table,
    Id,
    Number,
    Units,
    Season,
    Year,
    RelatedUrls,
    SpecialPermission,
    Description,
    Prerequisites,
    Corequisites,
    Crosslisted,
    Notes,
}

#[derive(Iden)]
enum Instructors {
    Table,
    Id,
    Name,
}

#[derive(Iden)]
enum Components {
    Table,
    Id,
    CourseId,
    Title,
    ComponentType,
    Code,
    SyllabusUrl,
}

#[derive(Iden)]
enum Meetings {
    Table,
    Id,
    ComponentId,
    DaysPattern,
    TimeBegin,
    TimeEnd,
    BldgRoom,
    Campus,
}

#[derive(Iden)]
enum InstructorMeetings {
    Table,
    Id,
    InstructorId,
    MeetingId,
}

#[derive(Iden)]
enum Reservations {
    Table,
    Id,
    StudentType,
    RestrictionType,
}

#[derive(Iden)]
enum ComponentReservations {
    Table,
    Id,
    ComponentId,
    ReservationId,
}

#[derive(Iden)]
enum Evaluations {
    Table,
    Id,
    ComponentId,
    InstructorId,
    CourseShortName,
    CourseLevel,
    TotalStudents,
    NumResponses,
    HoursPerWeek,
    InterestInStudentLearning,
    ClearlyExplainRequirements,
    ClearLearningObjectives,
    InstructorProvidesFeedback,
    DemonstrateImportance,
    ExplainsSubjectMatter,
    ShowRespectForStudents,
    OverallTeachingRate,
    OverallCourseRate,
}
