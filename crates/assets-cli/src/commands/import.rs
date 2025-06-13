use anyhow::Result;
use assets_core::importers::BoursoBankImporter;
use assets_core::{Database, ImportService};
use clap::{Args, Subcommand};
use uuid::Uuid;

#[derive(Subcommand)]
pub enum ImportCommands {
    /// Import BoursoBank CSV transactions
    Boursobank(BoursoBankArgs),
}

#[derive(Args)]
pub struct BoursoBankArgs {
    /// Path to the CSV file to import
    #[arg(short, long)]
    file: String,

    /// Target account path (e.g., "Assets:Current Assets:BoursoBank")
    #[arg(short, long)]
    account: String,

    /// User ID
    #[arg(short, long)]
    user_id: String,
}

pub async fn handle_import_command(command: ImportCommands) -> Result<()> {
    match command {
        ImportCommands::Boursobank(args) => import_boursobank(args).await,
    }
}

async fn import_boursobank(args: BoursoBankArgs) -> Result<()> {
    println!("💰 Importing BoursoBank Transactions");
    println!("====================================\n");

    let db = Database::from_env().await?;
    let user_id = Uuid::parse_str(&args.user_id)?;

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
