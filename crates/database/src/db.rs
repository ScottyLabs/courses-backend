use sea_orm::{Database, DatabaseConnection, DbErr};

/// Creates a database connection
pub async fn create_connection() -> Result<DatabaseConnection, DbErr> {
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");

    Database::connect(database_url).await
}
