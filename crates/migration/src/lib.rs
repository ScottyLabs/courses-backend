pub use sea_orm_migration::prelude::*;

mod m20250709_add_indexes;
mod m20250709_create_all_tables;
mod m20250710_add_component_title_index;
mod m20250710_add_search_indexes;
mod m20250710_create_tsvector_columns;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250709_create_all_tables::Migration),
            Box::new(m20250709_add_indexes::Migration),
            Box::new(m20250710_add_component_title_index::Migration),
            Box::new(m20250710_add_search_indexes::Migration),
            Box::new(m20250710_create_tsvector_columns::Migration),
        ]
    }
}
