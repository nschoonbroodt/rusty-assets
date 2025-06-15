use anyhow::Result;
use clap::{Parser, Subcommand};
use rust_decimal::Decimal;

mod commands;
#[cfg(feature = "demo")]
use commands::demo::*;
use commands::{
    accounts::*, db::*, duplicates::*, import::*, prices, reports::*, transactions::*, users::*,
};

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
    /// Account management and chart of accounts
    Accounts {
        #[command(subcommand)]
        action: AccountCommands,
    },
    /// Price tracking for investments
    Prices {
        #[command(subcommand)]
        action: PriceCommands,
    },
    /// Financial reports and analysis
    Reports {
        #[command(subcommand)]
        action: ReportCommands,
    },
    /// User management
    Users {
        #[command(subcommand)]
        action: UserCommands,
    },
    /// Transaction management and viewing
    Transactions {
        #[command(subcommand)]
        action: TransactionCommands,
    },
    /// Import bank transactions
    Import {
        #[command(subcommand)]
        action: ImportCommands,
    },
    /// Duplicate transaction detection and management
    Duplicates {
        #[command(subcommand)]
        action: DuplicateCommands,
    },
    #[cfg(feature = "demo")]
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
enum AccountCommands {
    /// List all accounts in a tree structure
    List,
    /// Show account balance and ownership details
    Balance {
        /// Account ID to show balance for
        #[arg(long)]
        id: Option<String>,
    },
    /// Create a new account interactively
    Create {
        /// Account name
        #[arg(long)]
        name: Option<String>,
        /// Account type (Asset, Liability, Equity, Income, Expense)
        #[arg(long)]
        account_type: Option<String>,
        /// Account subtype
        #[arg(long)]
        subtype: Option<String>,
        /// Parent account path (e.g., "Assets:Current Assets")
        #[arg(long)]
        parent: Option<String>,
        /// Stock/ETF symbol (for investment accounts)
        #[arg(long)]
        symbol: Option<String>,
        /// Currency code (default: EUR)
        #[arg(long, default_value = "EUR")]
        currency: String,
        /// Account notes
        #[arg(long)]
        notes: Option<String>,
    },
    /// Show chart of accounts as a tree
    Tree,
    /// Show account ownership details
    Ownership {
        /// Account ID to show ownership for
        account_id: String,
    },
    /// Set opening balance for an account
    SetOpeningBalance {
        /// Account path (e.g., "Assets:Current Assets:BoursoBank")
        account_path: String,
        /// Opening balance amount
        #[arg(long, allow_hyphen_values = true)]
        amount: Decimal,
        /// Date for the opening balance (default: January 1st of current year)
        #[arg(long)]
        date: Option<chrono::NaiveDate>,
        /// User who owns this account (default: lookup first user)
        #[arg(long)]
        user: Option<String>,
    },
}

#[derive(Subcommand)]
enum PriceCommands {
    /// Add a price entry for an asset
    Add,
    /// Show price history for a symbol or all symbols
    History {
        /// Symbol to show history for (optional)
        symbol: Option<String>,
    },
    /// Show market values for all investment accounts
    Market,
}

#[derive(Subcommand)]
enum ReportCommands {
    /// Generate balance sheet report
    BalanceSheet {
        #[command(flatten)]
        params: BalanceSheetParams,
    },
    /// Generate income statement report
    IncomeStatement {
        #[command(flatten)]
        params: IncomeStatementParams,
    },
    /// Generate cash flow statement
    CashFlow {
        #[command(flatten)]
        params: CashFlowParams,
    },
    /// Generate trial balance report
    TrialBalance {
        #[command(flatten)]
        params: TrialBalanceParams,
    },
    /// Generate account ledger report
    AccountLedger {
        #[command(flatten)]
        params: AccountLedgerParams,
    },
    /// Generate net worth report over time
    NetWorth {
        #[command(flatten)]
        params: NetWorthParams,
    },
    /// Generate budget vs actual report
    Budget {
        #[command(flatten)]
        params: BudgetReportParams,
    },
    /// Generate expense analysis report
    ExpenseAnalysis {
        #[command(flatten)]
        params: ExpenseAnalysisParams,
    },
    /// Generate investment performance report
    InvestmentPerformance {
        #[command(flatten)]
        params: InvestmentPerformanceParams,
    },
}

#[cfg(feature = "demo")]
#[derive(Subcommand)]
enum DemoCommands {
    // /// Demonstrate double-entry bookkeeping examples
    // DoubleEntry,
    // /// Show account types and their normal balance behavior
    // AccountTypes,
    // /// Multi-user examples with shared ownership
    // MultiUser,
    // /// Show ownership examples
    // Ownership,
    // /// Demonstrate nested category hierarchies
    // Categories,
    // /// Create sample users and accounts with database
    // CreateSample,
    // /// Create deep category hierarchy examples in database
    // CreateDeepCategories,
    // /// Create deep account hierarchy examples in database
    // CreateDeepAccounts,
    // /// Create sample price data for investments
    // CreateSamplePrices,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();
    env_logger::init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Db { action } => match action {
            DbCommands::Init => init_database().await?,
            DbCommands::Status => show_db_status().await?,
        },
        Commands::Accounts { action } => match action {
            AccountCommands::List => list_accounts().await?,
            AccountCommands::Balance { id } => show_account_balance(id.as_deref()).await?,
            AccountCommands::Create {
                name,
                account_type,
                subtype,
                parent,
                symbol,
                currency,
                notes,
            } => {
                create_account(
                    name.as_deref(),
                    account_type.as_deref(),
                    subtype.as_deref(),
                    parent.as_deref(),
                    symbol.as_deref(),
                    &currency,
                    notes.as_deref(),
                )
                .await?
            }
            AccountCommands::Tree => show_accounts_tree().await?,
            AccountCommands::Ownership { account_id } => {
                show_account_ownership(&account_id).await?
            }
            AccountCommands::SetOpeningBalance {
                account_path,
                amount,
                date,
                user,
            } => set_account_opening_balance(&account_path, amount, date, user.as_deref()).await?,
        },
        Commands::Prices { action } => match action {
            PriceCommands::Add => prices::add_price_interactive().await?,
            PriceCommands::History { symbol } => {
                prices::show_price_history(symbol.as_deref()).await?
            }
            PriceCommands::Market => prices::show_market_values().await?,
        },
        Commands::Reports { action } => match action {
            ReportCommands::BalanceSheet { params } => {
                generate_balance_sheet(params).await?;
            }
            ReportCommands::IncomeStatement { params } => {
                generate_income_statement(params).await?;
            }
            ReportCommands::CashFlow { params } => {
                generate_cash_flow_statement(params).await?;
            }
            ReportCommands::TrialBalance { params } => {
                generate_trial_balance(params).await?;
            }
            ReportCommands::AccountLedger { params } => {
                generate_account_ledger(params).await?;
            }
            ReportCommands::NetWorth { params } => {
                generate_net_worth_report(params).await?;
            }
            ReportCommands::Budget { params } => {
                generate_budget_report(params).await?;
            }
            ReportCommands::ExpenseAnalysis { params } => {
                generate_expense_analysis(params).await?;
            }
            ReportCommands::InvestmentPerformance { params } => {
                generate_investment_performance(params).await?;
            }
        },
        Commands::Users { action } => handle_user_command(action).await?,
        Commands::Transactions { action } => handle_transaction_command(action).await?,
        Commands::Import { action } => handle_import_command(action).await?,
        Commands::Duplicates { action } => handle_duplicate_command(action).await?,
        #[cfg(feature = "demo")]
        Commands::Demo { action } => match action {
            // DemoCommands::DoubleEntry => demo_double_entry().await?,
            // DemoCommands::AccountTypes => show_account_types(),
            // DemoCommands::MultiUser => show_multi_user_examples(),
            // DemoCommands::Ownership => show_ownership_examples(),
            // DemoCommands::Categories => show_category_examples().await?,
            // DemoCommands::CreateSample => create_sample_data(&cli.user).await?,
            // DemoCommands::CreateDeepCategories => create_deep_categories().await?,
            // DemoCommands::CreateDeepAccounts => create_deep_accounts().await?,
            // DemoCommands::CreateSamplePrices => create_sample_prices().await?,
        },
    }
    Ok(())
}
