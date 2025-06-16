use anyhow::Result;
use assets_core::{Database, UserService};
use clap::ValueEnum;
use uuid::Uuid;

/// Output format for reports
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    /// Display as a formatted table (default)
    Table,
    /// Export as JSON
    Json,
    /// Export as CSV
    Csv,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Table
    }
}

/// Helper function to get user UUID from username
pub async fn get_user_id_by_name(username: &str) -> Result<Uuid> {
    let db = Database::from_env().await?;
    let user_service = UserService::new(db.pool().clone());

    match user_service.get_user_by_name(username).await? {
        Some(user) => Ok(user.id),
        None => Err(anyhow::anyhow!("User '{}' not found", username)),
    }
}
