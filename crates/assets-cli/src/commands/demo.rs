use anyhow::Result;
use assets_core::{AccountType, Database, SampleDataService, TransactionService};
use chrono::Utc;
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

pub async fn demo_double_entry() -> Result<()> {
    println!("🏦 RustyAssets - Double-Entry Bookkeeping Demo");
    println!("===============================================\n");

    println!("Double-entry bookkeeping ensures every transaction balances:");
    println!("- Every transaction has multiple journal entries");
    println!("- Debits must equal credits (sum = 0)");
    println!("- Assets and Expenses increase with debits (+)");
    println!("- Liabilities, Equity, and Income increase with credits (-)\n");

    // Example transactions
    println!("📝 Example Transactions:\n"); // Example 1: Getting paid salary
    println!("1. Salary Payment: €3,000");
    let salary_transaction = TransactionService::create_simple_transaction(
        "Monthly salary payment".to_string(),
        Uuid::new_v4(), // Checking account (Asset)
        Uuid::new_v4(), // Salary income account
        Decimal::from_str("3000.00")?,
        Utc::now(),
        Some("PAY-2025-01".to_string()),
        None, // No specific user for demo
    );

    println!("   Debit:  Checking Account    +€3,000.00");
    println!("   Credit: Salary Income       +€3,000.00");
    println!(
        "   Balance check: €{:.2} (should be 0.00) ✓\n",
        salary_transaction
            .entries
            .iter()
            .map(|e| e.amount)
            .sum::<Decimal>()
    ); // Example 2: Buying groceries
    println!("2. Grocery Purchase: €150");
    let grocery_transaction = TransactionService::create_simple_transaction(
        "Weekly groceries".to_string(),
        Uuid::new_v4(), // Groceries expense account
        Uuid::new_v4(), // Credit card account (Liability)
        Decimal::from_str("150.00")?,
        Utc::now(),
        None,
        None, // No specific user for demo
    );

    println!("   Debit:  Groceries Expense   +€150.00");
    println!("   Credit: Credit Card         +€150.00 (liability)");
    println!(
        "   Balance check: €{:.2} (should be 0.00) ✓\n",
        grocery_transaction
            .entries
            .iter()
            .map(|e| e.amount)
            .sum::<Decimal>()
    ); // Example 3: Investment purchase with trading fee (3 journal entries)
    println!("3. Stock Purchase with Trading Fee: €2,500 + €9.99 fee");

    // Create a transaction with multiple journal entries manually
    use assets_core::{NewJournalEntry, NewTransaction};
    let stock_price = Decimal::from_str("2500.00")?;
    let trading_fee = Decimal::from_str("9.99")?;
    let total_cost = stock_price + trading_fee;    let investment_transaction = NewTransaction {
        description: "Purchase 15 shares of AAPL with trading fee".to_string(),
        reference: Some("TXN-20250611-001".to_string()),
        transaction_date: Utc::now(),
        created_by: None,
        entries: vec![
            // Debit: AAPL Stock (Asset increases)
            NewJournalEntry {
                account_id: Uuid::new_v4(), // AAPL stock account
                amount: stock_price,        // +€2,500.00
                memo: Some("15 shares of AAPL @ €166.67".to_string()),
            },
            // Debit: Trading Fee Expense (Expense increases)
            NewJournalEntry {
                account_id: Uuid::new_v4(), // Trading fees expense account
                amount: trading_fee,        // +€9.99
                memo: Some("Brokerage trading fee".to_string()),            },
            // Credit: Checking Account (Asset decreases)
            NewJournalEntry {
                account_id: Uuid::new_v4(), // Checking account
                amount: -total_cost,        // -€2,509.99
                memo: Some("Stock purchase payment".to_string()),
            },
        ],
        import_source: None,
        import_batch_id: None,
        external_reference: None,
    };

    println!("   Debit:  AAPL Stock          +€2,500.00");
    println!("   Debit:  Trading Fee Expense +€9.99");
    println!("   Credit: Checking Account    -€2,509.99");
    println!(
        "   Balance check: €{:.2} (should be 0.00) ✓\n",
        investment_transaction
            .entries
            .iter()
            .map(|e| e.amount)
            .sum::<Decimal>()
    );
    println!("🎯 Key Benefits:");
    println!("- Complete audit trail: see exactly where money comes from and goes");
    println!("- Built-in validation: transactions must balance");
    println!("- Multi-entry transactions: handle complex scenarios like fees, taxes, splits");
    println!("- Professional reporting: can generate balance sheets, income statements");
    println!("- Unified system: cash, investments, real estate are all accounts");
    println!("- Future-proof: easy to add new account types and features\n");

    Ok(())
}

pub fn show_account_types() {
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
        println!(
            "   {:12} {:10} {}",
            format!("{:?}:", acc_type),
            sign,
            meaning
        );
    }
}

pub fn show_multi_user_examples() {
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
    println!("   Apartment: You 64%, Spouse 36% (€400k total)");
    println!("   Mortgage:  You 50%, Spouse 50% (€200k debt)");
    println!("   Checking:  You 100% (individual account)");
    println!("   Savings:   Spouse 100% (individual account)\n");
    println!("📊 Balance Views:");
    println!("   Family View:  Apartment €400k, Mortgage -€200k = Net €200k");
    println!("   Your View:    Apartment €256k, Mortgage -€100k = Net €156k");
    println!("   Spouse View:  Apartment €144k, Mortgage -€100k = Net €44k\n");

    println!("🔄 Example Transaction Attribution:");
    println!("   You buy groceries (€150) → Attributed to you");
    println!("   Spouse pays utilities (€200) → Attributed to spouse");
    println!("   Mortgage payment (€1500) → Could be split or attributed to one person\n");

    println!("🎯 Benefits:");
    println!("- Fair ownership tracking without complex splitting");
    println!("- Each person sees their contribution and net worth");
    println!("- Combined family view for overall financial health");
    println!("- Transparent when switching between contexts");
}

pub fn show_ownership_examples() {
    println!("🏠 Fractional Ownership Examples");
    println!("================================\n");

    println!("Real estate and large assets often have fractional ownership between");
    println!("partners. Our system tracks this precisely without complex calculations.\n");
    println!("📋 Example 1: Apartment Purchase");
    println!("   Purchase Price: €400,000");
    println!("   You contribute: €256,000 (64%)");
    println!("   Spouse contributes: €144,000 (36%)");
    println!("   \n   Journal Entries:");
    println!("   Debit:  Real Estate - Apartment    +€400,000");
    println!("   Credit: Your Checking              -€256,000");
    println!("   Credit: Spouse Checking            -€144,000");
    println!("   \n   Ownership recorded: You 64%, Spouse 36%\n");

    println!("📋 Example 2: Mortgage (Shared Liability)");
    println!("   Loan Amount: €200,000");
    println!("   You responsible: €100,000 (50%)");
    println!("   Spouse responsible: €100,000 (50%)");
    println!("   \n   Journal Entries:");
    println!("   Debit:  Cash/Checking              +€200,000");
    println!("   Credit: Mortgage Payable           -€200,000");
    println!("   \n   Ownership recorded: You 50%, Spouse 50%\n");

    println!("📋 Example 3: Monthly Mortgage Payment");
    println!("   Payment: €1,500 (€1,200 principal + €300 interest)");
    println!("   Paid by: You (but affects both owners proportionally)");
    println!("   \n   Journal Entries:");
    println!("   Debit:  Mortgage Payable           +€1,200 (reduces debt)");
    println!("   Debit:  Interest Expense           +€300");
    println!("   Credit: Your Checking              -€1,500");
    println!("   \n   Effect: Your debt reduced by €600, Spouse's by €600\n");

    println!("🎯 Key Features:");
    println!("- Ownership percentages stored once, applied automatically");
    println!("- Balance sheets show proportional ownership per user");
    println!("- No need to manually split every transaction");
    println!("- Works for any asset: real estate, businesses, investments");
    println!("- Easy to change ownership percentages if needed");
}

pub async fn create_sample_data(user_context: &str) -> Result<()> {
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

pub async fn show_category_examples() -> Result<()> {
    println!("🗂️  Category Hierarchy Examples");
    println!("===============================\n");

    println!("📁 The category system supports UNLIMITED nesting levels!");
    println!("Each category can have a parent_id pointing to another category.\n");

    println!("🏠 Example: Home Expenses Hierarchy");
    println!("├── Home & Living");
    println!("│   ├── Utilities");
    println!("│   │   ├── Electricity");
    println!("│   │   ├── Gas");
    println!("│   │   ├── Water");
    println!("│   │   └── Internet");
    println!("│   ├── Maintenance");
    println!("│   │   ├── Plumbing");
    println!("│   │   ├── Electrical");
    println!("│   │   └── HVAC");
    println!("│   └── Decoration");
    println!("│       ├── Furniture");
    println!("│       │   ├── Living Room");
    println!("│       │   │   ├── Sofa");
    println!("│       │   │   ├── Coffee Table");
    println!("│       │   │   └── TV Stand");
    println!("│       │   ├── Bedroom");
    println!("│       │   │   ├── Bed Frame");
    println!("│       │   │   ├── Mattress");
    println!("│       │   │   └── Dresser");
    println!("│       │   └── Kitchen");
    println!("│       │       ├── Bar Stools");
    println!("│       │       └── Kitchen Island");
    println!("│       ├── Lighting");
    println!("│       │   ├── Ceiling Fixtures");
    println!("│       │   ├── Table Lamps");
    println!("│       │   └── Floor Lamps");
    println!("│       └── Artwork");
    println!("│           ├── Paintings");
    println!("│           ├── Sculptures");
    println!("│           └── Photography\n");

    println!("🚗 Example: Transportation Hierarchy");
    println!("├── Transportation");
    println!("│   ├── Vehicle Expenses");
    println!("│   │   ├── Fuel");
    println!("│   │   │   ├── Gasoline");
    println!("│   │   │   ├── Diesel");
    println!("│   │   │   └── Electric Charging");
    println!("│   │   ├── Maintenance");
    println!("│   │   │   ├── Oil Changes");
    println!("│   │   │   ├── Tire Replacement");
    println!("│   │   │   ├── Brake Service");
    println!("│   │   │   └── Inspections");
    println!("│   │   └── Insurance");
    println!("│   │       ├── Liability");
    println!("│   │       ├── Comprehensive");
    println!("│   │       └── Collision");
    println!("│   └── Public Transport");
    println!("│       ├── Subway/Metro");
    println!("│       ├── Bus");
    println!("│       ├── Train");
    println!("│       └── Rideshare");
    println!("│           ├── Uber");
    println!("│           ├── Lyft");
    println!("│           └── Taxi\n");

    println!("💡 How to create nested categories:");
    println!("1. Create the top-level category (parent_id = NULL)");
    println!("2. Create subcategories with parent_id pointing to the parent");
    println!("3. Create sub-subcategories with parent_id pointing to the subcategory");
    println!("4. Continue nesting as deep as needed!\n");

    println!("📝 SQL Example for 'Expense->Home->Deco->Furniture->Sofa':");
    println!("```sql");
    println!("-- 1. Create top level");
    println!("INSERT INTO categories (name) VALUES ('Expense');");
    println!("");
    println!("-- 2. Create Home under Expense");
    println!("INSERT INTO categories (name, parent_id) ");
    println!("VALUES ('Home', (SELECT id FROM categories WHERE name = 'Expense'));");
    println!("");
    println!("-- 3. Create Deco under Home");
    println!("INSERT INTO categories (name, parent_id)");
    println!("VALUES ('Deco', (SELECT id FROM categories WHERE name = 'Home'));");
    println!("");
    println!("-- 4. Create Furniture under Deco");
    println!("INSERT INTO categories (name, parent_id)");
    println!("VALUES ('Furniture', (SELECT id FROM categories WHERE name = 'Deco'));");
    println!("");
    println!("-- 5. Create Sofa under Furniture");
    println!("INSERT INTO categories (name, parent_id)");
    println!("VALUES ('Sofa', (SELECT id FROM categories WHERE name = 'Furniture'));");
    println!("```\n");

    println!("🎯 Benefits of Deep Hierarchies:");
    println!("- Precise expense tracking (know exactly what you spent on)");
    println!("- Flexible reporting (can roll up to any level)");
    println!("- Easy filtering (show all furniture expenses, or just sofas)");
    println!("- Inheritance (subcategories can inherit colors from parents)");
    println!("- Future-proof (add new levels without changing the structure)\n");

    println!("🔍 Querying Hierarchies:");
    println!("- Direct children: WHERE parent_id = <category_id>");
    println!("- All descendants: Use recursive CTE (Common Table Expression)");
    println!("- Full path: Join categories to itself multiple times");
    println!("- Breadcrumb navigation: Walk up the parent_id chain\n");

    println!("💼 Real-world use cases:");
    println!("- Business: Department -> Team -> Project -> Task -> Subtask");
    println!("- Shopping: Store -> Department -> Category -> Brand -> Product");
    println!("- Taxes: Tax Year -> Form -> Schedule -> Line Item -> Deduction");
    println!("- Investments: Portfolio -> Asset Class -> Sector -> Company -> Security");

    Ok(())
}

pub async fn create_deep_categories() -> Result<()> {
    println!("🗂️  Creating Deep Category Hierarchies");
    println!("======================================\n");

    // Connect to database
    let db = Database::from_env().await?;
    println!("✅ Connected to database");

    // Create sample data service
    let sample_service = SampleDataService::new(db);

    // Create deep hierarchies
    sample_service.create_deep_category_hierarchy().await?;

    println!("\n🎉 Deep category hierarchies created!");
    println!("\n📋 What was created:");
    println!("   • Expense → Home → Deco → Furniture → Sofa (5 levels deep)");
    println!("   • Transportation → Vehicle Expenses → Fuel → Gasoline → Premium Gas");
    println!("\n💡 Try querying with SQL:");
    println!("   SELECT name, parent_id FROM categories WHERE name = 'Sofa';");
    println!("   -- This will show that Sofa has Furniture as its parent");

    Ok(())
}

pub async fn create_deep_accounts() -> Result<()> {
    println!("🏦 Creating Deep Account Hierarchies");
    println!("=====================================\n");

    // Connect to database
    let db = Database::from_env().await?;
    println!("✅ Connected to database");

    // Create sample data service
    let sample_service = SampleDataService::new(db);

    // Create deep account hierarchies
    sample_service.create_deep_account_hierarchy().await?;

    println!("\n🎉 Deep account hierarchies created!");
    println!("\n📋 What was created:");
    println!("   • Assets → Bank1 → Cash, Savings, Brokerage");
    println!("   • Assets → Bank1 → Brokerage → AAPL, MSFT, GOOGL, SPY");
    println!("   • Assets → Bank2 → Checking, Money Market");
    println!("   • Assets → Investment Accounts → 401k, IRA");
    println!("   • Assets → Investment Accounts → 401k → Bond Fund, Stock Fund");
    println!("   • Assets → Real Estate → Primary Residence, Investment Properties");
    println!("   • Assets → Real Estate → Investment Properties → Rental Property 1, 2");
    println!("\n💡 Try viewing the tree structure:");
    println!("   cargo run -- accounts tree");
    println!("   -- This will show the nested hierarchy with proper indentation");

    Ok(())
}

pub async fn create_sample_prices() -> Result<()> {
    println!("📈 Creating Sample Price Data");
    println!("=============================\n");

    // Connect to database
    let db = Database::from_env().await?;
    println!("✅ Connected to database");

    // Create sample data service
    let sample_service = SampleDataService::new(db);

    // Create sample price data
    sample_service.create_sample_price_data().await?;

    println!("\n🎉 Sample price data created!");
    println!("\n💡 Try these commands to explore the price data:");
    println!("   cargo run -- prices history          # Show all tracked symbols");
    println!("   cargo run -- prices history AAPL     # Show AAPL price history");
    println!("   cargo run -- prices add              # Add a new price entry");
    println!("   cargo run -- prices market           # Show market values for investments");

    Ok(())
}
