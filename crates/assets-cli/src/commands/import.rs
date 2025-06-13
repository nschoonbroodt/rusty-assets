use anyhow::Result;
use assets_core::importers::{BoursoBankImporter, SocietegeneraleImporter};
use assets_core::{Database, ImportService, UserService};
use clap::{Args, Subcommand};
use uuid::Uuid;

/// Helper function to get user UUID from username
async fn get_user_id_by_name(username: &str) -> Result<Uuid> {
    let db = Database::from_env().await?;
    let user_service = UserService::new(db.pool().clone());

    match user_service.get_user_by_name(username).await? {
        Some(user) => Ok(user.id),
        None => Err(anyhow::anyhow!("User '{}' not found", username)),
    }
}

#[derive(Subcommand)]
pub enum ImportCommands {
    /// Import BoursoBank CSV transactions
    Boursobank(BoursoBankArgs),
    /// Import Société Générale CSV transactions
    Sg(SgArgs),
}

#[derive(Args)]
pub struct BoursoBankArgs {
    /// Path to the CSV file to import
    #[arg(short, long)]
    file: String,
    /// Target account path (e.g., "Assets:Current Assets:BoursoBank")
    #[arg(short, long)]
    account: String,

    /// Username (instead of UUID)
    #[arg(short, long)]
    user: String,
}

#[derive(Args)]
pub struct SgArgs {
    /// Path to the CSV file to import
    #[arg(short, long)]
    file: String,
    /// Target account path (e.g., "Assets:Current Assets:SG")
    #[arg(short, long)]
    account: String,

    /// Username (instead of UUID)
    #[arg(short, long)]
    user: String,
}

pub async fn handle_import_command(command: ImportCommands) -> Result<()> {
    match command {
        ImportCommands::Boursobank(args) => import_boursobank(args).await,
        ImportCommands::Sg(args) => import_sg(args).await,
    }
}

async fn import_boursobank(args: BoursoBankArgs) -> Result<()> {
    println!("💰 Importing BoursoBank Transactions");
    println!("====================================\n");

    let db = Database::from_env().await?;
    let user_id = get_user_id_by_name(&args.user).await?;

    let import_service = ImportService::new(db.pool().clone());
    let importer = BoursoBankImporter::new(args.account.clone());

    let summary = import_service
        .import_transactions(&importer, &args.file, &args.account, user_id)
        .await?;

    summary.print_summary();

    if summary.created > 0 {
        println!("\n✅ Import completed successfully!");
        println!("💡 Tip: Run 'assets-cli reports balance-sheet' to see your updated balance");
    }

    Ok(())
}

async fn import_sg(args: SgArgs) -> Result<()> {
    println!("🏦 Importing Société Générale Transactions");
    println!("==========================================\n");

    let db = Database::from_env().await?;
    let user_id = get_user_id_by_name(&args.user).await?;

    let import_service = ImportService::new(db.pool().clone());
    let importer = SocietegeneraleImporter::new(args.account.clone());

    let summary = import_service
        .import_transactions(&importer, &args.file, &args.account, user_id)
        .await?;

    summary.print_summary();

    if summary.created > 0 {
        println!("\n✅ Import completed successfully!");
        println!("💡 Tip: Run 'assets-cli reports balance-sheet' to see your updated balance");
    }

    Ok(())
}
