use anyhow::Result;
use assets_core::Database;
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

// User-related functions removed as ownership model has been eliminated
