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
mod tests;
