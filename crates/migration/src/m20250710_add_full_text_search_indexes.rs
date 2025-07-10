use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // TSVector indexes for full-text search (@@)
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX IF NOT EXISTS idx_courses_description_fts
                 ON courses USING gin (to_tsvector('english', COALESCE(description, '')));",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX IF NOT EXISTS idx_components_title_fts
                 ON components USING gin (to_tsvector('english', title));",
            )
            .await?;

        // B-tree index for exact course number matches
        manager
            .create_index(
                Index::create()
                    .name("idx_courses_number_exact")
                    .table(Courses::Table)
                    .col(Courses::Number)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_courses_number_exact").to_owned())
            .await?;

        manager
            .get_connection()
            .execute_unprepared("DROP INDEX IF EXISTS idx_components_title_fts;")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("DROP INDEX IF EXISTS idx_courses_description_fts;")
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Courses {
    Table,
    Number,
}
