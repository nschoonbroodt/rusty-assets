use anyhow::Result;
use assets_core::{
    services::AccountService, AccountSubtype, AccountType, Database, NewAccountByPath,
    NewTransactionByPath,
};
use chrono::{Datelike, Utc};
use log::{debug, error, info};
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
    info!("💰 Creating sample transactions...");

    let account_service = AccountService::new(db.pool().clone());
    let transaction_service = TransactionService::new(db.pool().clone());

    // Calculate dates for previous month and current month
    let now = Utc::now();
    let current_month_start = now.with_day(1).unwrap();
    let previous_month_start = current_month_start
        .checked_sub_months(chrono::Months::new(1))
        .unwrap();

    info!(
        "📅 Creating transactions for {} and {}",
        previous_month_start.format("%Y-%m"),
        current_month_start.format("%Y-%m")
    );

    // Previous Month Transactions
    let prev_month_transactions = vec![
        // May 1: Salary
        (
            1,
            "Monthly Salary Deposit",
            NewTransactionByPath::income(
                "Monthly Salary Deposit",
                previous_month_start.with_day(1).unwrap(),
                "Income:Employment:Salary",
                "Assets:Current Assets:Main Checking",
                Decimal::new(320000, 2), // €3,200.00
            ),
        ),
        // May 2: Rent
        (
            2,
            "Rent Payment",
            NewTransactionByPath::expense(
                "Rent Payment",
                previous_month_start.with_day(2).unwrap(),
                "Expenses:Housing:Rent",
                "Assets:Current Assets:Main Checking",
                Decimal::new(120000, 2), // €1,200.00
            ),
        ),
        // May 3: Groceries
        (
            3,
            "Groceries - Carrefour",
            NewTransactionByPath::expense(
                "Groceries - Carrefour",
                previous_month_start.with_day(3).unwrap(),
                "Expenses:Food:Groceries",
                "Assets:Current Assets:Main Checking",
                Decimal::new(8550, 2), // €85.50
            ),
        ),
        // May 5: Electric Bill
        (
            5,
            "Electric Bill - EDF",
            NewTransactionByPath::expense(
                "Electric Bill - EDF",
                previous_month_start.with_day(5).unwrap(),
                "Expenses:Utilities:Electric",
                "Assets:Current Assets:Main Checking",
                Decimal::new(6820, 2), // €68.20
            ),
        ),
        // May 7: Gas
        (
            7,
            "Gas Station - Total",
            NewTransactionByPath::expense(
                "Gas Station - Total",
                previous_month_start.with_day(7).unwrap(),
                "Expenses:Transportation:Gas",
                "Assets:Current Assets:Main Checking",
                Decimal::new(5500, 2), // €55.00
            ),
        ),
        // May 10: Groceries
        (
            10,
            "Groceries - Monoprix",
            NewTransactionByPath::expense(
                "Groceries - Monoprix",
                previous_month_start.with_day(10).unwrap(),
                "Expenses:Food:Groceries",
                "Assets:Current Assets:Main Checking",
                Decimal::new(9230, 2), // €92.30
            ),
        ),
        // May 12: Internet
        (
            12,
            "Internet Bill - Orange",
            NewTransactionByPath::expense(
                "Internet Bill - Orange",
                previous_month_start.with_day(12).unwrap(),
                "Expenses:Utilities:Internet",
                "Assets:Current Assets:Main Checking",
                Decimal::new(3599, 2), // €35.99
            ),
        ),
        // May 15: Dining Out
        (
            15,
            "Dinner - Bistrot",
            NewTransactionByPath::expense(
                "Dinner - Bistrot",
                previous_month_start.with_day(15).unwrap(),
                "Expenses:Food:Dining Out",
                "Assets:Current Assets:Main Checking",
                Decimal::new(4580, 2), // €45.80
            ),
        ),
        // May 20: Emergency Fund Transfer
        (
            20,
            "Emergency Fund Transfer",
            NewTransactionByPath::simple_transfer(
                "Emergency Fund Transfer",
                previous_month_start.with_day(20).unwrap(),
                "Assets:Current Assets:Main Checking",
                "Assets:Current Assets:Emergency Fund",
                Decimal::new(50000, 2), // €500.00
            ),
        ),
        // May 25: Phone Bill
        (
            25,
            "Phone Bill - SFR",
            NewTransactionByPath::expense(
                "Phone Bill - SFR",
                previous_month_start.with_day(25).unwrap(),
                "Expenses:Utilities:Phone",
                "Assets:Current Assets:Main Checking",
                Decimal::new(2599, 2), // €25.99
            ),
        ),
    ];

    // Current Month Transactions
    let current_month_transactions = vec![
        // June 1: Salary
        (
            1,
            "Monthly Salary Deposit",
            NewTransactionByPath::income(
                "Monthly Salary Deposit",
                current_month_start.with_day(1).unwrap(),
                "Income:Employment:Salary",
                "Assets:Current Assets:Main Checking",
                Decimal::new(320000, 2), // €3,200.00
            ),
        ),
        // June 2: Rent
        (
            2,
            "Rent Payment",
            NewTransactionByPath::expense(
                "Rent Payment",
                current_month_start.with_day(2).unwrap(),
                "Expenses:Housing:Rent",
                "Assets:Current Assets:Main Checking",
                Decimal::new(120000, 2), // €1,200.00
            ),
        ),
        // June 4: Groceries
        (
            4,
            "Groceries - Leclerc",
            NewTransactionByPath::expense(
                "Groceries - Leclerc",
                current_month_start.with_day(4).unwrap(),
                "Expenses:Food:Groceries",
                "Assets:Current Assets:Main Checking",
                Decimal::new(7845, 2), // €78.45
            ),
        ),
        // June 8: Gas
        (
            8,
            "Gas Station - Shell",
            NewTransactionByPath::expense(
                "Gas Station - Shell",
                current_month_start.with_day(8).unwrap(),
                "Expenses:Transportation:Gas",
                "Assets:Current Assets:Main Checking",
                Decimal::new(6200, 2), // €62.00
            ),
        ),
        // June 10: Car Insurance
        (
            10,
            "Car Insurance - Quarterly",
            NewTransactionByPath::expense(
                "Car Insurance - Quarterly",
                current_month_start.with_day(10).unwrap(),
                "Expenses:Transportation:Car Insurance",
                "Assets:Current Assets:Main Checking",
                Decimal::new(8500, 2), // €85.00
            ),
        ),
        // June 12: Groceries
        (
            12,
            "Groceries - Carrefour",
            NewTransactionByPath::expense(
                "Groceries - Carrefour",
                current_month_start.with_day(12).unwrap(),
                "Expenses:Food:Groceries",
                "Assets:Current Assets:Main Checking",
                Decimal::new(8920, 2), // €89.20
            ),
        ),
        // June 14: Clothing
        (
            14,
            "Clothing - Zara",
            NewTransactionByPath::expense(
                "Clothing - Zara",
                current_month_start.with_day(14).unwrap(),
                "Expenses:Personal:Clothing",
                "Assets:Current Assets:Main Checking",
                Decimal::new(12000, 2), // €120.00
            ),
        ),
        // June 15: Entertainment
        (
            15,
            "Cinema - UGC",
            NewTransactionByPath::expense(
                "Cinema - UGC",
                current_month_start.with_day(15).unwrap(),
                "Expenses:Personal:Entertainment",
                "Assets:Current Assets:Main Checking",
                Decimal::new(3550, 2), // €35.50
            ),
        ),
    ];

    // Create previous month transactions
    info!(
        "📅 Creating {} previous month transactions...",
        prev_month_transactions.len()
    );
    for (day, desc, transaction) in prev_month_transactions {
        transaction_service
            .create_transaction_by_path(&account_service, transaction)
            .await
            .map_err(|e| {
                error!(
                    "❌ Failed to create transaction '{}' (day {}): {}",
                    desc, day, e
                );
                e
            })?;
        debug!("✅ Created: {}", desc);
    }

    // Create current month transactions
    info!(
        "📅 Creating {} current month transactions...",
        current_month_transactions.len()
    );
    for (day, desc, transaction) in current_month_transactions {
        transaction_service
            .create_transaction_by_path(&account_service, transaction)
            .await
            .map_err(|e| {
                error!(
                    "❌ Failed to create transaction '{}' (day {}): {}",
                    desc, day, e
                );
                e
            })?;
        debug!("✅ Created: {}", desc);
    }

    info!("✅ All sample transactions created successfully!");
    Ok(())
}
