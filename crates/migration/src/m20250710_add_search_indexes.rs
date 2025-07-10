use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Composite index for common filter combinations (season + year + course number prefix)
        manager
            .create_index(
                Index::create()
                    .name("idx_courses_season_year_number")
                    .table(Courses::Table)
                    .col(Courses::Season)
                    .col(Courses::Year)
                    .col(Courses::Number)
                    .to_owned(),
            )
            .await?;

        // Index on course description for better text search performance
        manager
            .create_index(
                Index::create()
                    .name("idx_courses_description")
                    .table(Courses::Table)
                    .col(Courses::Description)
                    .to_owned(),
            )
            .await?;

        // Composite index for instructor searches
        manager
            .create_index(
                Index::create()
                    .name("idx_instructor_meetings_composite")
                    .table(InstructorMeetings::Table)
                    .col(InstructorMeetings::InstructorId)
                    .col(InstructorMeetings::MeetingId)
                    .to_owned(),
            )
            .await?;

        // Index for special permission courses
        manager
            .create_index(
                Index::create()
                    .name("idx_courses_special_permission")
                    .table(Courses::Table)
                    .col(Courses::SpecialPermission)
                    .to_owned(),
            )
            .await?;

        // Enable the pg_trgm extension for trigram-based text search
        manager
            .get_connection()
            .execute_unprepared("CREATE EXTENSION IF NOT EXISTS pg_trgm;")
            .await?;

        // GIN index on course description for fuzzy text search
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX IF NOT EXISTS idx_courses_description_gin
                 ON courses USING gin (description gin_trgm_ops);",
            )
            .await?;

        // GIN index on component title for fuzzy text search
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX IF NOT EXISTS idx_components_title_gin
                 ON components USING gin (title gin_trgm_ops);",
            )
            .await?;

        // GIN index on course number for partial matching
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX IF NOT EXISTS idx_courses_number_gin
                 ON courses USING gin (number gin_trgm_ops);",
            )
            .await?;

        // Expression index for department code (first 2 digits)
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX IF NOT EXISTS idx_courses_department_code
                 ON courses (substring(number from 1 for 2));",
            )
            .await?;

        // Composite GIN index for multi-column text search
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX IF NOT EXISTS idx_courses_full_text_search
                 ON courses USING gin ((number || ' ' || COALESCE(description, '')) gin_trgm_ops);",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop PostgreSQL-specific indexes first
        manager
            .get_connection()
            .execute_unprepared("DROP INDEX IF EXISTS idx_courses_full_text_search;")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("DROP INDEX IF EXISTS idx_courses_department_code;")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("DROP INDEX IF EXISTS idx_courses_number_gin;")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("DROP INDEX IF EXISTS idx_components_title_gin;")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("DROP INDEX IF EXISTS idx_courses_description_gin;")
            .await?;

        // Drop standard indexes
        manager
            .drop_index(
                Index::drop()
                    .name("idx_courses_special_permission")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_instructor_meetings_composite")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(Index::drop().name("idx_courses_description").to_owned())
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_courses_season_year_number")
                    .to_owned(),
            )
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
    Description,
    SpecialPermission,
}

#[derive(Iden)]
enum InstructorMeetings {
    Table,
    InstructorId,
    MeetingId,
}
