use anyhow::Result;
use assets_core::{AccountType, TransactionService};
use chrono::Utc;
use clap::{Parser, Subcommand};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "assets-cli")]
#[command(about = "RustyAssets - Personal Finance Tracker with Double-Entry Bookkeeping")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Demonstrate double-entry bookkeeping examples
    Demo,
    /// Show account types and their normal balance behavior
    AccountTypes,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Demo => demo_double_entry().await?,
        Commands::AccountTypes => show_account_types(),
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
