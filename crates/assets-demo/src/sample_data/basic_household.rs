use anyhow::Result;
use assets_core::{
    services::AccountService, AccountSubtype, AccountType, Database, NewAccountByPath,
    NewTransactionByPath,
};
use chrono::{Datelike, TimeZone};
use log::{error, info};
use rust_decimal::Decimal;

pub async fn create_basic_household_demo() -> Result<()> {
    info!("🏠 Creating basic household demo data...");

    info!("🔌 Connecting to database and ensuring clean state...");
    let db = Database::from_env().await.map_err(|e| {
        error!("❌ Failed to connect to database: {}", e);
        e
    })?;

    info!("🚀 Applying migrations...");
    db.migrate().await.map_err(|e| {
        error!("❌ Failed to apply migrations: {}", e);
        e
    })?;

    info!("📊 Creating the account structure...");
    create_accounts(db.clone()).await.map_err(|e| {
        error!("❌ Failed to create accounts: {}", e);
        e
    })?;

    info!("💰 Creating sample transactions...");
    create_transactions(&db).await.map_err(|e| {
        error!("❌ Failed to create transactions: {}", e);
        e
    })?;

    info!("🎉 Basic household demo created successfully! Try running 'cargo run -- accounts tree' to see your new accounts.");
    Ok(())
}

async fn create_accounts(db: Database) -> Result<()> {
    let account_service = AccountService::new(db.pool().clone());

    #[rustfmt::skip]
    let accounts= vec![
        ("Assets:Current Assets:Main Checking", AccountType::Asset, AccountSubtype::Checking),
        ("Assets:Current Assets:Savings Account", AccountType::Asset, AccountSubtype::Savings),
        ("Assets:Current Assets:Emergency Fund", AccountType::Asset, AccountSubtype::Savings),
        ("Liabilities:Credit Cards:Visa Card", AccountType::Liability, AccountSubtype::CreditCard),
        ("Income:Employment:Salary", AccountType::Income, AccountSubtype::Salary),
        ("Expenses:Food:Dining Out", AccountType::Expense, AccountSubtype::Food),
        ("Expenses:Food:Groceries", AccountType::Expense, AccountSubtype::Food),
        ("Expenses:Housing:Rent", AccountType::Expense, AccountSubtype::Housing),
        ("Expenses:Personal:Clothing", AccountType::Expense, AccountSubtype::Personal),
        ("Expenses:Personal:Entertainment", AccountType::Expense, AccountSubtype::Personal),
        ("Expenses:Transportation:Car Insurance", AccountType::Expense, AccountSubtype::Transportation),
        ("Expenses:Transportation:Gas", AccountType::Expense, AccountSubtype::Transportation),
        ("Expenses:Utilities:Electric", AccountType::Expense, AccountSubtype::Utilities),
        ("Expenses:Utilities:Internet", AccountType::Expense, AccountSubtype::Utilities),
        ("Expenses:Utilities:Phone", AccountType::Expense, AccountSubtype::Utilities),
    ];

    for (path, account_type, account_subtype) in accounts {
        account_service
            .create_account_by_path(
                NewAccountByPath::builder()
                    .full_path(path)
                    .account_type(account_type)
                    .account_subtype(account_subtype)
                    .build(),
            )
            .await
            .map_err(|e| {
                error!("❌ Failed to create account '{}': {}", path, e);
                e
            })?;
    }

    info!("✅ Complete chart of accounts created successfully!");

    Ok(())
}

use assets_core::services::TransactionService;

async fn create_transactions(db: &Database) -> Result<()> {
    let now = chrono::Utc::now();
    let previous_month = now - chrono::Duration::days(30);
    let previous_month_start = chrono::Utc
        .with_ymd_and_hms(
            previous_month.year(),
            previous_month.month(),
            1, // First day of the month
            0,
            0,
            0,
        )
        .unwrap();

    let account_service = AccountService::new(db.pool().clone());
    let transaction_service = TransactionService::new(db.pool().clone());

    // Calculate dates: previous month (1st) and current month (1st)
    info!(
        "📅 Creating transactions for previous month starting: {}",
        previous_month_start
    );

    let salary_transaction = NewTransactionByPath::income(
        "Monthly Salary Deposit",
        previous_month_start,
        "Income:Employment:Salary",
        "Assets:Current Assets:Main Checking",
        Decimal::new(320000, 2), // €3,200.00
    );

    transaction_service
        .create_transaction_by_path(&account_service, salary_transaction)
        .await
        .map_err(|e| {
            error!("❌ Failed to create salary transaction: {}", e);
            e
        })?;

    info!("✅ Sample transactions created successfully!");

    Ok(())
}
