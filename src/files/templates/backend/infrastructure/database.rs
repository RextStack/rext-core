use sea_orm::*;
use sqlx::SqlitePool;
use std::env;
use std::time::Duration;

/// Database connection manager
pub struct DatabaseManager;

impl DatabaseManager {
    /// Creates and configures the database connection
    pub async fn create_connection() -> Result<DatabaseConnection, Box<dyn std::error::Error>> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

        let mut opts = ConnectOptions::new(database_url.clone());

        // Enable SQLx logging for query performance tracking
        opts.sqlx_logging(true)
            .max_connections(20)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(8))
            .acquire_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8));

        let db: DatabaseConnection = Database::connect(opts)
            .await
            .expect("Failed to connect to database");

        println!("Connected to database: {}", database_url);
        Ok(db)
    }

    /// Creates a SQLite pool for job queue operations
    pub async fn create_pool() -> Result<SqlitePool, Box<dyn std::error::Error>> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");
        let pool = sqlx::SqlitePool::connect(&database_url).await?;
        Ok(pool)
    }

    /// Sets up job queue storage tables
    pub async fn setup_job_queue_storage(
        pool: &SqlitePool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use apalis_sql::sqlite::SqliteStorage;

        SqliteStorage::setup(pool)
            .await
            .expect("unable to run migrations for sqlite");

        Ok(())
    }
}
