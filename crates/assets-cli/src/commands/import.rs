use anyhow::Result;
use assets_core::importers::{
    BoursoBankImporter, GenericPayslipImporter, QtPayslipImporter, SocietegeneraleImporter,
};
use assets_core::{Database, ImportService, PayslipImportService, UserService};
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
    /// Import payslip data
    Payslip(PayslipArgs),
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

#[derive(Args)]
pub struct PayslipArgs {
    /// Path to the payslip file to import
    #[arg(short, long)]
    file: String,
    /// Target account path for net salary (e.g., "Assets:Current Assets:BoursoBank")
    #[arg(short, long)]
    account: String,
    /// Username (instead of UUID)
    #[arg(short, long)]
    user: String,
    /// Importer type (default: generic)
    #[arg(long, default_value = "generic")]
    importer: String,
}

pub async fn handle_import_command(command: ImportCommands) -> Result<()> {
    match command {
        ImportCommands::Boursobank(args) => import_boursobank(args).await,
        ImportCommands::Sg(args) => import_sg(args).await,
        ImportCommands::Payslip(args) => import_payslip(args).await,
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

async fn import_payslip(args: PayslipArgs) -> Result<()> {
    println!("💰 Importing Payslip");
    println!("====================\n");

    let db = Database::from_env().await?;
    let payslip_import_service = PayslipImportService::new(db.pool().clone());
    let result = match args.importer.as_str() {
        "generic" => {
            let importer = GenericPayslipImporter::new();
            payslip_import_service
                .import_payslip(&importer, &args.file, &args.account, &args.user)
                .await?
        }
        "qt" => {
            let importer = QtPayslipImporter::new();
            payslip_import_service
                .import_payslip(&importer, &args.file, &args.account, &args.user)
                .await?
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown payslip importer: {}. Available: generic, qt",
                args.importer
            ));
        }
    };

    // Print import summary
    println!("📊 Import Summary");
    println!("=================");
    println!("• Pay Date: {}", result.payslip_info.pay_date);
    println!(
        "• Pay Period: {} to {}",
        result.payslip_info.pay_period_start, result.payslip_info.pay_period_end
    );
    println!("• Employee: {}", result.payslip_info.employee_name);
    println!("• Employer: {}", result.payslip_info.employer_name);
    println!("• Gross Salary: €{}", result.payslip_info.gross_salary);
    println!("• Net Salary: €{}", result.payslip_info.net_salary);
    println!("• Line Items: {}", result.payslip_info.line_items_count);
    println!("• Transaction ID: {}", result.transaction_id);

    if !result.warnings.is_empty() {
        println!("\n⚠️  Warnings:");
        for warning in &result.warnings {
            println!("  • {}", warning);
        }
    }

    println!("\n✅ Payslip import completed successfully!");
    println!("💡 Tip: Run 'assets-cli reports balance-sheet' to see your updated balance");
    println!(
        "💡 Tip: Run 'assets-cli reports income-statement --user {}' to see income details",
        args.user
    );

    Ok(())
}
