use dotenv_codegen::dotenv;
use sea_orm::{Database, DatabaseConnection, DbErr};

const DATABASE_URL: &str = dotenv!("DATABASE_URL");

/// Creates a database connection
pub async fn create_connection() -> Result<DatabaseConnection, DbErr> {
    Database::connect(DATABASE_URL).await
}
