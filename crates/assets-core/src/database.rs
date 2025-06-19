use crate::Result;
use sqlx::{PgPool, postgres::PgPoolOptions};

#[derive(Debug, Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Create a new database connection from a connection string
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Create a new database connection from environment variable
    pub async fn from_env() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL")?;
        Self::new(&database_url).await
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;
    use testcontainers_modules::{
        postgres,
        testcontainers::{ImageExt, runners::AsyncRunner},
    };

    #[tokio::test]
    async fn test_database_migrations() {
        // Start PostgreSQL container with contrib extensions
        let container = postgres::Postgres::default()
            .with_tag("15") // Use PostgreSQL 15 which should have uuid support
            .start()
            .await
            .unwrap();
        let host_port = container.get_host_port_ipv4(5432).await.unwrap();
        let database_url = format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            host_port
        );

        // Create database connection
        let database = Database::new(&database_url)
            .await
            .expect("Failed to connect to test database");

        // Create required extensions manually (since test container might not have them)
        sqlx::query("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\"")
            .execute(database.pool())
            .await
            .expect("Failed to create uuid-ossp extension");

        sqlx::query("CREATE EXTENSION IF NOT EXISTS pg_trgm")
            .execute(database.pool())
            .await
            .expect("Failed to create pg_trgm extension");

        // Run migrations
        database.migrate().await.expect("Failed to run migrations");

        // Verify core tables exist
        let tables = sqlx::query(
            "SELECT table_name FROM information_schema.tables 
             WHERE table_schema = 'public'
             ORDER BY table_name",
        )
        .fetch_all(database.pool())
        .await
        .expect("Failed to query tables");

        let table_names: Vec<String> = tables
            .iter()
            .map(|row| {
                row.get::<Option<String>, _>("table_name")
                    .unwrap_or_default()
            })
            .collect();

        // Verify essential tables exist
        assert!(
            table_names.contains(&"users".to_string()),
            "Users table missing"
        );
        assert!(
            table_names.contains(&"accounts".to_string()),
            "Accounts table missing"
        );
        assert!(
            table_names.contains(&"transactions".to_string()),
            "Transactions table missing"
        );
        assert!(
            table_names.contains(&"journal_entries".to_string()),
            "Journal entries table missing"
        );

        // Verify we can query tables (basic connectivity test)
        let table_count = sqlx::query(
            "SELECT COUNT(*) as count FROM information_schema.tables 
             WHERE table_schema = 'public'",
        )
        .fetch_one(database.pool())
        .await
        .expect("Failed to count tables");

        let count: i64 = table_count.get("count");
        assert!(count > 0, "No tables found after migration");

        println!(
            "✅ Migration test passed! Created {} tables",
            table_names.len()
        );
    }

    #[tokio::test]
    async fn test_database_from_env() {
        // Start PostgreSQL container
        let container = postgres::Postgres::default()
            .with_tag("15")
            .start()
            .await
            .unwrap();
        let host_port = container.get_host_port_ipv4(5432).await.unwrap();
        let database_url = format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            host_port
        );

        // Set environment variable
        unsafe {
            std::env::set_var("DATABASE_URL", &database_url);
        }

        // Test from_env() method
        let database = Database::from_env()
            .await
            .expect("Failed to connect using from_env()");

        // Simple connectivity test
        let result = sqlx::query("SELECT 1 as test")
            .fetch_one(database.pool())
            .await
            .expect("Failed to execute test query");

        let test_value: i32 = result.get("test");
        assert_eq!(test_value, 1, "Test query should return 1");

        println!("✅ from_env() test passed!");

        // Clean up environment variable
        unsafe {
            std::env::remove_var("DATABASE_URL");
        }
    }
}
