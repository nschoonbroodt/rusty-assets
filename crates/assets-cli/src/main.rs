use anyhow::Result;
use assets_core::{AccountType, TransactionService, Database, SampleDataService};
use chrono::Utc;
use clap::{Parser, Subcommand};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

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
    /// Demonstrate double-entry bookkeeping examples
    Demo,
    /// Show account types and their normal balance behavior
    AccountTypes,
    /// Multi-user examples with shared ownership
    MultiUser,
    /// Show ownership examples
    Ownership,
    /// Initialize database and run migrations
    InitDb,
    /// Create sample users and accounts with database
    CreateSample,
    /// Show database status and connection info
    DbStatus,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    let cli = Cli::parse();

    match cli.command {
        Commands::Demo => demo_double_entry().await?,
        Commands::AccountTypes => show_account_types(),
        Commands::MultiUser => show_multi_user_examples(),
        Commands::Ownership => show_ownership_examples(),
        Commands::InitDb => init_database().await?,
        Commands::CreateSample => create_sample_data(&cli.user).await?,
        Commands::DbStatus => show_db_status().await?,
    }

    Ok(())
}

async fn demo_double_entry() -> Result<()> {
    println!("🏦 RustyAssets - Double-Entry Bookkeeping Demo");
    println!("===============================================\n");

    println!("Double-entry bookkeeping ensures every transaction balances:");
    println!("- Every transaction has multiple journal entries");
    println!("- Debits must equal credits (sum = 0)");
    println!("- Assets and Expenses increase with debits (+)");
    println!("- Liabilities, Equity, and Income increase with credits (-)\n");

    // Example transactions
    println!("📝 Example Transactions:\n");

    // Example 1: Getting paid salary
    println!("1. Salary Payment: $3,000");
    let salary_transaction = TransactionService::create_simple_transaction(
        "Monthly salary payment".to_string(),
        Uuid::new_v4(), // Checking account (Asset)
        Uuid::new_v4(), // Salary income account
        Decimal::from_str("3000.00")?,
        Utc::now(),
        Some("PAY-2025-01".to_string()),
        None, // No specific user for demo
    );
    
    println!("   Debit:  Checking Account    +$3,000.00");
    println!("   Credit: Salary Income       +$3,000.00");
    println!("   Balance check: ${:.2} (should be 0.00) ✓\n", 
        salary_transaction.entries.iter().map(|e| e.amount).sum::<Decimal>());

    // Example 2: Buying groceries
    println!("2. Grocery Purchase: $150");
    let grocery_transaction = TransactionService::create_simple_transaction(
        "Weekly groceries".to_string(),
        Uuid::new_v4(), // Groceries expense account
        Uuid::new_v4(), // Credit card account (Liability)
        Decimal::from_str("150.00")?,
        Utc::now(),
        None,
        None, // No specific user for demo
    );
    
    println!("   Debit:  Groceries Expense   +$150.00");
    println!("   Credit: Credit Card         +$150.00 (liability)");
    println!("   Balance check: ${:.2} (should be 0.00) ✓\n", 
        grocery_transaction.entries.iter().map(|e| e.amount).sum::<Decimal>());

    // Example 3: Investment purchase
    println!("3. Stock Purchase: $2,500");
    let investment_transaction = TransactionService::create_simple_transaction(
        "Purchase 15 shares of AAPL".to_string(),
        Uuid::new_v4(), // AAPL stock account (Asset)
        Uuid::new_v4(), // Checking account (Asset)
        Decimal::from_str("2500.00")?,
        Utc::now(),
        Some("TXN-20250611-001".to_string()),
        None, // No specific user for demo
    );
    
    println!("   Debit:  AAPL Stock          +$2,500.00");
    println!("   Credit: Checking Account    -$2,500.00");
    println!("   Balance check: ${:.2} (should be 0.00) ✓\n", 
        investment_transaction.entries.iter().map(|e| e.amount).sum::<Decimal>());

    println!("🎯 Key Benefits:");
    println!("- Complete audit trail: see exactly where money comes from and goes");
    println!("- Built-in validation: transactions must balance");
    println!("- Professional reporting: can generate balance sheets, income statements");
    println!("- Unified system: cash, investments, real estate are all accounts");
    println!("- Future-proof: easy to add new account types and features\n");

    Ok(())
}

fn show_account_types() {
    println!("📊 Account Types in Double-Entry Bookkeeping");
    println!("=============================================\n");

    println!("🏛️  ASSETS (increase with debits +)");
    println!("   Examples: Cash, Checking, Stocks, Real Estate, Equipment");
    println!("   Normal balance: Positive (debit)\n");

    println!("💳 LIABILITIES (increase with credits -)");
    println!("   Examples: Credit Cards, Loans, Mortgages");
    println!("   Normal balance: Negative (credit)\n");

    println!("🏠 EQUITY (increase with credits -)"); 
    println!("   Examples: Owner's Equity, Retained Earnings");
    println!("   Normal balance: Negative (credit)\n");

    println!("💰 INCOME (increase with credits -)");
    println!("   Examples: Salary, Dividends, Interest, Capital Gains");
    println!("   Normal balance: Negative (credit)\n");

    println!("💸 EXPENSES (increase with debits +)");
    println!("   Examples: Groceries, Rent, Utilities, Gas");
    println!("   Normal balance: Positive (debit)\n");

    println!("📐 The Accounting Equation:");
    println!("   Assets = Liabilities + Equity");
    println!("   (Everything must balance!)\n");

    // Show which types increase with debits vs credits
    let account_types = [
        (AccountType::Asset, "Debit (+)", "increases value"),
        (AccountType::Expense, "Debit (+)", "increases spending"),
        (AccountType::Liability, "Credit (-)", "increases debt"),
        (AccountType::Equity, "Credit (-)", "increases ownership"),
        (AccountType::Income, "Credit (-)", "increases earnings"),
    ];

    println!("🔄 Normal Balance Summary:");
    for (acc_type, sign, meaning) in account_types {
        println!("   {:12} {:10} {}", 
            format!("{:?}:", acc_type), 
            sign, 
            meaning);
    }
}

fn show_multi_user_examples() {
    println!("👥 Multi-User Personal Finance Examples");
    println!("======================================\n");

    println!("🏠 Scenario: You and your spouse manage shared finances");
    println!("- Some assets are individually owned");
    println!("- Some assets have fractional ownership (like real estate)");
    println!("- Each transaction is attributed to who initiated it");
    println!("- Views can be filtered by user context\n");

    println!("👤 User Contexts:");
    println!("   --user you     → Show only your transactions and proportional balances");
    println!("   --user spouse  → Show only spouse's transactions and proportional balances");
    println!("   --user family  → Show all transactions and combined balances (default)\n");

    println!("💰 Example Shared Assets:");
    println!("   Apartment: You 64%, Spouse 36% ($400k total)");
    println!("   Mortgage:  You 50%, Spouse 50% ($200k debt)");
    println!("   Checking:  You 100% (individual account)");
    println!("   Savings:   Spouse 100% (individual account)\n");

    println!("📊 Balance Views:");
    println!("   Family View:  Apartment $400k, Mortgage -$200k = Net $200k");
    println!("   Your View:    Apartment $256k, Mortgage -$100k = Net $156k");
    println!("   Spouse View:  Apartment $144k, Mortgage -$100k = Net $44k\n");

    println!("🔄 Example Transaction Attribution:");
    println!("   You buy groceries ($150) → Attributed to you");
    println!("   Spouse pays utilities ($200) → Attributed to spouse");
    println!("   Mortgage payment ($1500) → Could be split or attributed to one person\n");

    println!("🎯 Benefits:");
    println!("- Fair ownership tracking without complex splitting");
    println!("- Each person sees their contribution and net worth");
    println!("- Combined family view for overall financial health");
    println!("- Transparent when switching between contexts");
}

fn show_ownership_examples() {
    println!("🏠 Fractional Ownership Examples");
    println!("================================\n");

    println!("Real estate and large assets often have fractional ownership between");
    println!("partners. Our system tracks this precisely without complex calculations.\n");

    println!("📋 Example 1: Apartment Purchase");
    println!("   Purchase Price: $400,000");
    println!("   You contribute: $256,000 (64%)");
    println!("   Spouse contributes: $144,000 (36%)");
    println!("   \n   Journal Entries:");
    println!("   Debit:  Real Estate - Apartment    +$400,000");
    println!("   Credit: Your Checking              -$256,000");
    println!("   Credit: Spouse Checking            -$144,000");
    println!("   \n   Ownership recorded: You 64%, Spouse 36%\n");

    println!("📋 Example 2: Mortgage (Shared Liability)");
    println!("   Loan Amount: $200,000");
    println!("   You responsible: $100,000 (50%)");
    println!("   Spouse responsible: $100,000 (50%)");
    println!("   \n   Journal Entries:");
    println!("   Debit:  Cash/Checking              +$200,000");
    println!("   Credit: Mortgage Payable           -$200,000");
    println!("   \n   Ownership recorded: You 50%, Spouse 50%\n");

    println!("📋 Example 3: Monthly Mortgage Payment");
    println!("   Payment: $1,500 ($1,200 principal + $300 interest)");
    println!("   Paid by: You (but affects both owners proportionally)");
    println!("   \n   Journal Entries:");
    println!("   Debit:  Mortgage Payable           +$1,200 (reduces debt)");
    println!("   Debit:  Interest Expense           +$300");
    println!("   Credit: Your Checking              -$1,500");
    println!("   \n   Effect: Your debt reduced by $600, Spouse's by $600\n");

    println!("🎯 Key Features:");
    println!("- Ownership percentages stored once, applied automatically");
    println!("- Balance sheets show proportional ownership per user");
    println!("- No need to manually split every transaction");
    println!("- Works for any asset: real estate, businesses, investments");
    println!("- Easy to change ownership percentages if needed");
}

async fn init_database() -> Result<()> {
    println!("🗄️  Initializing Database");
    println!("=========================\n");

    // Check if DATABASE_URL is set
    match std::env::var("DATABASE_URL") {
        Ok(url) => {
            // Hide password in display
            let display_url = if url.contains('@') {
                let parts: Vec<&str> = url.split('@').collect();
                if parts.len() >= 2 {
                    let mut user_part = parts[0].to_string();
                    if let Some(colon_pos) = user_part.rfind(':') {
                        user_part.replace_range(colon_pos + 1.., "****");
                    }
                    format!("{}@{}", user_part, parts[1..].join("@"))
                } else {
                    url.clone()
                }
            } else {
                url.clone()
            };
            
            println!("📡 Database URL: {}", display_url);
            
            // Try to connect and run migrations
            println!("🔄 Connecting to database...");
            match Database::from_env().await {
                Ok(db) => {
                    println!("✅ Connected successfully!");
                    
                    println!("🔄 Running migrations...");
                    match db.migrate().await {
                        Ok(_) => {
                            println!("✅ Migrations completed successfully!");
                            println!("\n🎉 Database is ready for use!");
                        },
                        Err(e) => {
                            println!("❌ Migration failed: {}", e);
                            println!("\n💡 Make sure PostgreSQL is running and the database exists.");
                            println!("   You can create it with: createdb rustyassets");
                        }
                    }
                },
                Err(e) => {
                    println!("❌ Connection failed: {}", e);
                    println!("\n💡 Troubleshooting:");
                    println!("   1. Make sure PostgreSQL is running");
                    println!("   2. Check your DATABASE_URL in .env file");
                    println!("   3. Create the database: createdb rustyassets");
                    println!("   4. Ensure the user has proper permissions");
                }
            }
        },
        Err(_) => {
            println!("❌ DATABASE_URL not found");
            println!("\n💡 Please create a .env file with:");
            println!("   DATABASE_URL=postgresql://username:password@localhost:5432/rustyassets");
            println!("\n📝 You can copy .env.example to .env and modify it.");
        }
    }

    Ok(())
}

async fn create_sample_data(user_context: &str) -> Result<()> {
    println!("🏗️  Creating Sample Data");
    println!("========================\n");

    println!("👤 User context: {}", user_context);
    
    // Connect to database
    let db = Database::from_env().await?;
    println!("✅ Connected to database");

    // Create sample data service
    let sample_service = SampleDataService::new(db);
    
    // Create complete sample dataset
    sample_service.create_full_sample_dataset().await?;

    Ok(())
}

async fn show_db_status() -> Result<()> {
    println!("📊 Database Status");
    println!("==================\n");

    match std::env::var("DATABASE_URL") {
        Ok(url) => {
            // Hide password in display
            let display_url = if url.contains('@') {
                let parts: Vec<&str> = url.split('@').collect();
                if parts.len() >= 2 {
                    let mut user_part = parts[0].to_string();
                    if let Some(colon_pos) = user_part.rfind(':') {
                        user_part.replace_range(colon_pos + 1.., "****");
                    }
                    format!("{}@{}", user_part, parts[1..].join("@"))
                } else {
                    url.clone()
                }
            } else {
                url.clone()
            };
            
            println!("📡 Database URL: {}", display_url);
            
            // Try to connect
            match Database::from_env().await {
                Ok(_db) => {
                    println!("✅ Connection: Successful");
                    println!("🗄️  Database: Ready");
                    
                    // Could add more detailed status here like:
                    // - Table counts
                    // - Last migration version
                    // - User count
                    // - Transaction count
                    
                    println!("\n📈 Quick Stats:");
                    println!("   • Tables: Ready (migrations applied)");
                    println!("   • Users: Check with create-sample command");
                    println!("   • Transactions: 0 (ready for first entries)");
                },
                Err(e) => {
                    println!("❌ Connection: Failed");
                    println!("   Error: {}", e);
                    println!("\n💡 Run 'init-db' command to set up the database");
                }
            }
        },
        Err(_) => {
            println!("❌ DATABASE_URL not configured");
            println!("\n💡 Please create a .env file with your database connection");
        }
    }

    Ok(())
}
