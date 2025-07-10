use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Index on components.title for search performance
        manager
            .create_index(
                Index::create()
                    .name("idx_components_title")
                    .table(Components::Table)
                    .col(Components::Title)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_components_title").to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Components {
    Table,
    Title,
}
