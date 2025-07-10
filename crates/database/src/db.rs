use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::time::Duration;

/// Creates a database connection
pub async fn create_connection() -> Result<DatabaseConnection, DbErr> {
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");

    let mut opt = ConnectOptions::new(database_url);

    // Connection pool settings
    opt.max_connections(20) // Maximum 20 connections in pool
        .min_connections(5) // Always keep 5 connections alive
        .connect_timeout(Duration::from_secs(10)) // Max time to get connection
        .acquire_timeout(Duration::from_secs(10)) // Max time to wait for available connection
        .idle_timeout(Duration::from_secs(300)) // Close idle connections after 5 min
        .max_lifetime(Duration::from_secs(1800)); // Recreate connections every 30 min

    Database::connect(opt).await
}
