use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add tsvector columns for better full-text search performance
        manager
            .alter_table(
                Table::alter()
                    .table(Courses::Table)
                    .add_column(
                        ColumnDef::new(Courses::SearchVector).custom(Alias::new("tsvector")),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Components::Table)
                    .add_column(
                        ColumnDef::new(Components::SearchVector).custom(Alias::new("tsvector")),
                    )
                    .to_owned(),
            )
            .await?;

        // Create function to update search vectors
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE OR REPLACE FUNCTION update_course_search_vector() RETURNS trigger AS $$
                BEGIN
                    NEW.search_vector := to_tsvector('english',
                        COALESCE(NEW.number, '') || ' ' ||
                        COALESCE(NEW.description, '')
                    );
                    RETURN NEW;
                END;
                $$ LANGUAGE plpgsql;",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "CREATE OR REPLACE FUNCTION update_component_search_vector() RETURNS trigger AS $$
                BEGIN
                    NEW.search_vector := to_tsvector('english', COALESCE(NEW.title, ''));
                    RETURN NEW;
                END;
                $$ LANGUAGE plpgsql;",
            )
            .await?;

        // Create triggers to automatically update search vectors
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TRIGGER courses_search_vector_update
                BEFORE INSERT OR UPDATE ON courses
                FOR EACH ROW EXECUTE FUNCTION update_course_search_vector();",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TRIGGER components_search_vector_update
                BEFORE INSERT OR UPDATE ON components
                FOR EACH ROW EXECUTE FUNCTION update_component_search_vector();",
            )
            .await?;

        // Populate existing data
        manager
            .get_connection()
            .execute_unprepared(
                "UPDATE courses SET search_vector = to_tsvector('english',
                    COALESCE(number, '') || ' ' || COALESCE(description, ''));",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "UPDATE components SET search_vector = to_tsvector('english', COALESCE(title, ''));"
            )
            .await?;

        // Create GIN indexes on the tsvector columns
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX idx_courses_search_vector ON courses USING gin(search_vector);",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX idx_components_search_vector ON components USING gin(search_vector);",
            )
            .await?;

        // Create index for component course_id lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_components_course_id_title")
                    .table(Components::Table)
                    .col(Components::CourseId)
                    .col(Components::Title)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop triggers
        manager
            .get_connection()
            .execute_unprepared("DROP TRIGGER IF EXISTS courses_search_vector_update ON courses;")
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "DROP TRIGGER IF EXISTS components_search_vector_update ON components;",
            )
            .await?;

        // Drop functions
        manager
            .get_connection()
            .execute_unprepared("DROP FUNCTION IF EXISTS update_course_search_vector();")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("DROP FUNCTION IF EXISTS update_component_search_vector();")
            .await?;

        // Drop indexes
        manager
            .get_connection()
            .execute_unprepared("DROP INDEX IF EXISTS idx_courses_search_vector;")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("DROP INDEX IF EXISTS idx_components_search_vector;")
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_components_course_id_title")
                    .to_owned(),
            )
            .await?;

        // Drop columns
        manager
            .alter_table(
                Table::alter()
                    .table(Courses::Table)
                    .drop_column(Courses::SearchVector)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Components::Table)
                    .drop_column(Components::SearchVector)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Courses {
    Table,
    SearchVector,
}

#[derive(Iden)]
enum Components {
    Table,
    CourseId,
    Title,
    SearchVector,
}
