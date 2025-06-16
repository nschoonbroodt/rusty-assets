use anyhow::Result;
use assets_core::{
    services::AccountService, AccountSubtype, AccountType, Database, NewAccountByPath,
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

    account_service
        .create_account_by_path(
            NewAccountByPath::builder()
                .full_path("Assets:Current Assets:Main Checking")
                .account_type(AccountType::Asset)
                .account_subtype(AccountSubtype::Checking)
                .build(),
        )
        .await?;

    account_service
        .create_account_by_path(
            NewAccountByPath::builder()
                .full_path("Assets:Current Assets:Savings Account")
                .account_type(AccountType::Asset)
                .account_subtype(AccountSubtype::Savings)
                .build(),
        )
        .await?;

    account_service
        .create_account_by_path(
            NewAccountByPath::builder()
                .full_path("Assets:Current Assets:Emergency Fund")
                .account_type(AccountType::Asset)
                .account_subtype(AccountSubtype::Savings)
                .build(),
        )
        .await?;

    // Create Liabilities
    account_service
        .create_account_by_path(
            NewAccountByPath::builder()
                .full_path("Liabilities:Credit Cards:Visa Card")
                .account_type(AccountType::Liability)
                .account_subtype(AccountSubtype::CreditCard)
                .build(),
        )
        .await?;

    // Create Income accounts
    account_service
        .create_account_by_path(
            NewAccountByPath::builder()
                .full_path("Income:Employment:Salary")
                .account_type(AccountType::Income)
                .account_subtype(AccountSubtype::Salary)
                .build(),
        )
        .await?;

    // Create Expense accounts
    account_service
        .create_account_by_path(
            NewAccountByPath::builder()
                .full_path("Expenses:Food:Dining Out")
                .account_type(AccountType::Expense)
                .account_subtype(AccountSubtype::Food)
                .build(),
        )
        .await?;
    account_service
        .create_account_by_path(
            NewAccountByPath::builder()
                .full_path("Expenses:Food:Groceries")
                .account_type(AccountType::Expense)
                .account_subtype(AccountSubtype::Food)
                .build(),
        )
        .await?;

    account_service
        .create_account_by_path(
            NewAccountByPath::builder()
                .full_path("Expenses:Housing:Rent")
                .account_type(AccountType::Expense)
                .account_subtype(AccountSubtype::Housing)
                .build(),
        )
        .await?;

    account_service
        .create_account_by_path(
            NewAccountByPath::builder()
                .full_path("Expenses:Personal:Clothing")
                .account_type(AccountType::Expense)
                .account_subtype(AccountSubtype::Personal)
                .build(),
        )
        .await?;
    account_service
        .create_account_by_path(
            NewAccountByPath::builder()
                .full_path("Expenses:Personal:Entertainment")
                .account_type(AccountType::Expense)
                .account_subtype(AccountSubtype::Personal)
                .build(),
        )
        .await?;
    account_service
        .create_account_by_path(
            NewAccountByPath::builder()
                .full_path("Expenses:Transportation:Car Insurance")
                .account_type(AccountType::Expense)
                .account_subtype(AccountSubtype::Transportation)
                .build(),
        )
        .await?;
    account_service
        .create_account_by_path(
            NewAccountByPath::builder()
                .full_path("Expenses:Transportation:Gas")
                .account_type(AccountType::Expense)
                .account_subtype(AccountSubtype::Transportation)
                .build(),
        )
        .await?;
    account_service
        .create_account_by_path(
            NewAccountByPath::builder()
                .full_path("Expenses:Utilities:Electric")
                .account_type(AccountType::Expense)
                .account_subtype(AccountSubtype::Utilities)
                .build(),
        )
        .await?;
    account_service
        .create_account_by_path(
            NewAccountByPath::builder()
                .full_path("Expenses:Utilities:Internet")
                .account_type(AccountType::Expense)
                .account_subtype(AccountSubtype::Utilities)
                .build(),
        )
        .await?;
    account_service
        .create_account_by_path(
            NewAccountByPath::builder()
                .full_path("Expenses:Utilities:Phone")
                .account_type(AccountType::Expense)
                .account_subtype(AccountSubtype::Utilities)
                .build(),
        )
        .await?;

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

    // Get account references
    let checking_account = account_service
        .get_account_by_path("Assets:Current Assets:Main Checking")
        .await?;
    let salary_account = account_service
        .get_account_by_path("Income:Employment:Salary")
        .await?;

    // Calculate dates: previous month (1st) and current month (1st)
    info!(
        "📅 Creating transactions for previous month starting: {}",
        previous_month_start
    );

    // First transaction: Monthly Salary Deposit (previous month)
    let salary_transaction = TransactionService::create_simple_transaction(
        "Monthly Salary Deposit".to_string(),
        checking_account.id,     // Debit (asset increases)
        salary_account.id,       // Credit (income increases)
        Decimal::new(320000, 2), // €3,200.00
        previous_month_start,
        None, // no reference
        None, // no user context for demo
    );

    transaction_service
        .create_transaction(salary_transaction)
        .await?;

    info!("✅ Sample transactions created successfully!");

    Ok(())
}
