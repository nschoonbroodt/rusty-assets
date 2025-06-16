use anyhow::Result;
use assets_core::{services::AccountService, AccountSubtype, AccountType, Database, NewAccount};
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
    let assets = account_service
        .create_account(
            NewAccount::builder()
                .name("Assets")
                .account_type(AccountType::Asset)
                .account_subtype(AccountSubtype::Category)
                .build(),
        )
        .await?;

    let current_assets = account_service
        .create_account(
            NewAccount::builder()
                .name("Current Assets")
                .account_type(AccountType::Asset)
                .account_subtype(AccountSubtype::Category)
                .parent_id(assets.id)
                .build(),
        )
        .await?;

    let _main_checking_account = account_service
        .create_account(
            NewAccount::builder()
                .name("Main Checking")
                .account_type(AccountType::Asset)
                .account_subtype(AccountSubtype::Checking)
                .parent_id(current_assets.id)
                .build(),
        )
        .await?;

    let _savings_account = account_service
        .create_account(
            NewAccount::builder()
                .name("Savings Account")
                .account_type(AccountType::Asset)
                .account_subtype(AccountSubtype::Savings)
                .parent_id(current_assets.id)
                .build(),
        )
        .await?;

    let _emergency_fund_account = account_service
        .create_account(
            NewAccount::builder()
                .name("Emergency Fund")
                .account_type(AccountType::Asset)
                .account_subtype(AccountSubtype::Savings)
                .parent_id(current_assets.id)
                .build(),
        )
        .await?;

    // Create Liabilities
    let liabilities = account_service
        .create_account(
            NewAccount::builder()
                .name("Liabilities")
                .account_type(AccountType::Liability)
                .account_subtype(AccountSubtype::Category)
                .build(),
        )
        .await?;

    // Create Credit Cards under Liabilities (not Assets!)
    let credit_cards = account_service
        .create_account(
            NewAccount::builder()
                .name("Credit Cards")
                .account_type(AccountType::Liability)
                .account_subtype(AccountSubtype::Category)
                .parent_id(liabilities.id)
                .build(),
        )
        .await?;
    let _visa_card = account_service
        .create_account(
            NewAccount::builder()
                .name("Visa Card")
                .account_type(AccountType::Liability)
                .account_subtype(AccountSubtype::CreditCard)
                .parent_id(credit_cards.id)
                .build(),
        )
        .await?;

    // Create Income accounts
    let income = account_service
        .create_account(
            NewAccount::builder()
                .name("Income")
                .account_type(AccountType::Income)
                .account_subtype(AccountSubtype::Category)
                .build(),
        )
        .await?;

    let employment = account_service
        .create_account(
            NewAccount::builder()
                .name("Employment")
                .account_type(AccountType::Income)
                .account_subtype(AccountSubtype::Category)
                .parent_id(income.id)
                .build(),
        )
        .await?;

    let _salary = account_service
        .create_account(
            NewAccount::builder()
                .name("Salary")
                .account_type(AccountType::Income)
                .account_subtype(AccountSubtype::Salary)
                .parent_id(employment.id)
                .build(),
        )
        .await?;

    // Create Expense accounts
    let expenses = account_service
        .create_account(
            NewAccount::builder()
                .name("Expenses")
                .account_type(AccountType::Expense)
                .account_subtype(AccountSubtype::Category)
                .build(),
        )
        .await?;

    // Housing expenses
    let housing = account_service
        .create_account(
            NewAccount::builder()
                .name("Housing")
                .account_type(AccountType::Expense)
                .account_subtype(AccountSubtype::Housing)
                .parent_id(expenses.id)
                .build(),
        )
        .await?;

    let _rent = account_service
        .create_account(
            NewAccount::builder()
                .name("Rent")
                .account_type(AccountType::Expense)
                .account_subtype(AccountSubtype::Housing)
                .parent_id(housing.id)
                .build(),
        )
        .await?;

    // Food expenses
    let food = account_service
        .create_account(
            NewAccount::builder()
                .name("Food")
                .account_type(AccountType::Expense)
                .account_subtype(AccountSubtype::Food)
                .parent_id(expenses.id)
                .build(),
        )
        .await?;

    let _groceries = account_service
        .create_account(
            NewAccount::builder()
                .name("Groceries")
                .account_type(AccountType::Expense)
                .account_subtype(AccountSubtype::Food)
                .parent_id(food.id)
                .build(),
        )
        .await?;

    let _dining_out = account_service
        .create_account(
            NewAccount::builder()
                .name("Dining Out")
                .account_type(AccountType::Expense)
                .account_subtype(AccountSubtype::Food)
                .parent_id(food.id)
                .build(),
        )
        .await?;

    // Transportation expenses
    let transportation = account_service
        .create_account(
            NewAccount::builder()
                .name("Transportation")
                .account_type(AccountType::Expense)
                .account_subtype(AccountSubtype::Transportation)
                .parent_id(expenses.id)
                .build(),
        )
        .await?;

    let _gas = account_service
        .create_account(
            NewAccount::builder()
                .name("Gas")
                .account_type(AccountType::Expense)
                .account_subtype(AccountSubtype::Transportation)
                .parent_id(transportation.id)
                .build(),
        )
        .await?;

    let _car_insurance = account_service
        .create_account(
            NewAccount::builder()
                .name("Car Insurance")
                .account_type(AccountType::Expense)
                .account_subtype(AccountSubtype::Transportation)
                .parent_id(transportation.id)
                .build(),
        )
        .await?;

    // Utilities expenses
    let utilities = account_service
        .create_account(
            NewAccount::builder()
                .name("Utilities")
                .account_type(AccountType::Expense)
                .account_subtype(AccountSubtype::Utilities)
                .parent_id(expenses.id)
                .build(),
        )
        .await?;

    let _electric = account_service
        .create_account(
            NewAccount::builder()
                .name("Electric")
                .account_type(AccountType::Expense)
                .account_subtype(AccountSubtype::Utilities)
                .parent_id(utilities.id)
                .build(),
        )
        .await?;

    let _internet = account_service
        .create_account(
            NewAccount::builder()
                .name("Internet")
                .account_type(AccountType::Expense)
                .account_subtype(AccountSubtype::Utilities)
                .parent_id(utilities.id)
                .build(),
        )
        .await?;

    let _phone = account_service
        .create_account(
            NewAccount::builder()
                .name("Phone")
                .account_type(AccountType::Expense)
                .account_subtype(AccountSubtype::Communication)
                .parent_id(utilities.id)
                .build(),
        )
        .await?;

    // Personal expenses
    let personal = account_service
        .create_account(
            NewAccount::builder()
                .name("Personal")
                .account_type(AccountType::Expense)
                .account_subtype(AccountSubtype::Personal)
                .parent_id(expenses.id)
                .build(),
        )
        .await?;

    let _entertainment = account_service
        .create_account(
            NewAccount::builder()
                .name("Entertainment")
                .account_type(AccountType::Expense)
                .account_subtype(AccountSubtype::Entertainment)
                .parent_id(personal.id)
                .build(),
        )
        .await?;

    let _clothing = account_service
        .create_account(
            NewAccount::builder()
                .name("Clothing")
                .account_type(AccountType::Expense)
                .account_subtype(AccountSubtype::Personal)
                .parent_id(personal.id)
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
