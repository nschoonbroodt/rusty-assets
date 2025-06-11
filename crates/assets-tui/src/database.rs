use anyhow::Result;
use assets_core::Database;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Singleton database connection for the TUI
#[derive(Clone)]
pub struct AppDatabase {
    db: Arc<Mutex<Option<Database>>>,
}

impl AppDatabase {
    /// Create a new database wrapper
    pub fn new() -> Self {
        Self {
            db: Arc::new(Mutex::new(None)),
        }
    }
    
    /// Initialize the database connection
    pub async fn init(&self) -> Result<()> {
        // Load environment variables from .env file
        dotenv::dotenv().ok();
        
        // Create a database connection
        let db = Database::from_env().await?;
        
        // Store the connection
        let mut lock = self.db.lock().await;
        *lock = Some(db);
        
        Ok(())
    }
    
    /// Get a reference to the database connection
    pub async fn get(&self) -> Result<Database> {
        let lock = self.db.lock().await;
        match &*lock {
            Some(db) => Ok(db.clone()),
            None => Err(anyhow::anyhow!("Database not initialized")),
        }
    }
}

/// Initialize the database connection from environment variables
pub async fn init_database() -> Result<Database> {
    dotenv::dotenv().ok();
    Ok(Database::from_env().await?)
}
