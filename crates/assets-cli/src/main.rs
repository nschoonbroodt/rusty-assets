use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
use commands::*;

#[derive(Parser)]
#[command(name = "assets-cli")]
#[command(about = "RustyAssets - Personal Finance Tracker with Double-Entry Bookkeeping")]
struct Cli {
    /// User context: 'you', 'spouse', or 'family' (default: family)
    #[arg(long, default_value = "family")]
    user: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Database operations
    Db {
        #[command(subcommand)]
        action: DbCommands,
    },
    /// Demo and examples
    Demo {
        #[command(subcommand)]
        action: DemoCommands,
    },
}

#[derive(Subcommand)]
enum DbCommands {
    /// Initialize database and run migrations
    Init,
    /// Show database status and connection info
    Status,
}

#[derive(Subcommand)]
enum DemoCommands {
    /// Demonstrate double-entry bookkeeping examples
    DoubleEntry,
    /// Show account types and their normal balance behavior
    AccountTypes,
    /// Multi-user examples with shared ownership
    MultiUser,
    /// Show ownership examples
    Ownership,
    /// Demonstrate nested category hierarchies
    Categories,
    /// Create sample users and accounts with database
    CreateSample,
    /// Create deep category hierarchy examples in database
    CreateDeepCategories,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    let cli = Cli::parse();

    match cli.command {
        Commands::Db { action } => match action {
            DbCommands::Init => init_database().await?,
            DbCommands::Status => show_db_status().await?,
        },
        Commands::Demo { action } => match action {
            DemoCommands::DoubleEntry => demo_double_entry().await?,
            DemoCommands::AccountTypes => show_account_types(),
            DemoCommands::MultiUser => show_multi_user_examples(),
            DemoCommands::Ownership => show_ownership_examples(),
            DemoCommands::Categories => show_category_examples().await?,
            DemoCommands::CreateSample => create_sample_data(&cli.user).await?,
            DemoCommands::CreateDeepCategories => create_deep_categories().await?,
        },
    }
    Ok(())
}
