use crate::database::Database;
use crate::error::Result;
use sqlx::Row;
use uuid::Uuid;

pub struct SampleDataService {
    db: Database,
}

impl SampleDataService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Create sample categories for expense tracking
    pub async fn create_sample_categories(&self) -> Result<()> {
        println!("📊 Creating sample categories...");

        // Main categories
        let categories = vec![
            ("Food & Dining", "#FF6B6B"),
            ("Transportation", "#4ECDC4"),
            ("Shopping", "#45B7D1"),
            ("Entertainment", "#96CEB4"),
            ("Bills & Utilities", "#FFEAA7"),
            ("Healthcare", "#DDA0DD"),
            ("Education", "#98D8C8"),
            ("Travel", "#F7DC6F"),
            ("Income", "#82E0AA"),
            ("Investments", "#AED6F1"),
            ("Savings", "#F8C471"),
            ("Other", "#D2B4DE"),
        ];

        for (name, color) in categories {
            sqlx::query(
                "INSERT INTO categories (id, name, color, is_active) 
                 VALUES ($1, $2, $3, $4) 
                 ON CONFLICT (name) DO NOTHING",
            )
            .bind(Uuid::new_v4())
            .bind(name)
            .bind(color)
            .bind(true)
            .execute(self.db.pool())
            .await?;
        }

        // Create subcategories for Food & Dining
        let food_category_id: Option<Uuid> =
            sqlx::query("SELECT id FROM categories WHERE name = 'Food & Dining'")
                .fetch_optional(self.db.pool())
                .await?
                .map(|row| row.get("id"));

        if let Some(parent_id) = food_category_id {
            let subcategories = vec![
                "Restaurants",
                "Groceries",
                "Coffee & Tea",
                "Fast Food",
                "Alcohol & Bars",
            ];

            for subcategory in subcategories {
                sqlx::query(
                    "INSERT INTO categories (id, name, parent_id, color, is_active) 
                     VALUES ($1, $2, $3, $4, $5) 
                     ON CONFLICT (name) DO NOTHING",
                )
                .bind(Uuid::new_v4())
                .bind(subcategory)
                .bind(parent_id)
                .bind("#FF8E8E")
                .bind(true)
                .execute(self.db.pool())
                .await?;
            }
        }

        println!("✅ Sample categories created");
        Ok(())
    }

    /// Create sample chart of accounts
    pub async fn create_sample_accounts(&self) -> Result<()> {
        println!("🏦 Creating sample chart of accounts...");

        // Assets (1000-1999)
        let asset_accounts = vec![
            // (Code, Name, AccountType, AccountSubtype)
            ("Checking Account", "asset", "checking"),
            ("Savings Account", "asset", "savings"),
            ("Cash", "asset", "cash"),
            ("Brokerage Account", "asset", "investment_account"),
            ("Apple Inc. (AAPL)", "asset", "stocks"),
            ("S&P 500 ETF (SPY)", "asset", "etf"),
            ("Bitcoin", "asset", "crypto"),
            ("Primary Residence", "asset", "real_estate"),
            ("Rental Property", "asset", "real_estate"),
        ];

        // Liabilities (2000-2999)
        let liability_accounts = vec![
            ("Credit Card", "liability", "credit_card"),
            ("Home Mortgage", "liability", "mortgage"),
            ("Car Loan", "liability", "loan"),
        ];

        // Equity (3000-3999)
        let equity_accounts = vec![
            ("Opening Balance Equity", "equity", "opening_balance"),
            ("Retained Earnings", "equity", "retained_earnings"),
        ];

        // Income (4000-4999)
        let income_accounts = vec![
            ("Salary", "income", "salary"),
            ("Bonus", "income", "bonus"),
            ("Investment Income", "income", "investment"),
            ("Rental Income", "income", "rental"),
        ];

        // Expenses (5000-5999)
        let expense_accounts = vec![
            ("Groceries", "expense", "food"),
            ("Restaurants", "expense", "food"),
            ("Gas", "expense", "transportation"),
            ("Car Maintenance", "expense", "transportation"),
            ("Rent", "expense", "housing"),
            ("Utilities", "expense", "housing"),
            ("Internet", "expense", "housing"),
            ("Phone", "expense", "communication"),
            ("Entertainment", "expense", "entertainment"),
            ("Clothing", "expense", "personal"),
        ];

        let all_accounts = [
            asset_accounts,
            liability_accounts,
            equity_accounts,
            income_accounts,
            expense_accounts,
        ]
        .concat();
        for (name, account_type, account_subtype) in all_accounts {
            sqlx::query(
                "INSERT INTO accounts (name, account_type, account_subtype, currency) 
                 VALUES ($1, $2::account_type, $3::account_subtype, $4) 
                 ON CONFLICT (name, parent_id) DO NOTHING", // Assuming (name, parent_id) is the new unique constraint for non-root accounts. Root accounts use just name.
                                                            // For simplicity in sample data, we'll assume all sample accounts are root accounts or have unique names globally if parent_id is NULL.
                                                            // A more robust sample data generator might need to handle parent_id explicitly if creating hierarchies.
            )
            // .bind(code) // Removed code binding
            .bind(name)
            .bind(account_type)
            .bind(account_subtype)
            .bind("USD") // Added default currency, adjust as needed
            .execute(self.db.pool())
            .await?;
        }

        println!("✅ Sample chart of accounts created");
        Ok(())
    }
    /// Create sample users
    pub async fn create_sample_users(&self) -> Result<()> {
        println!("👥 Creating sample users...");

        let users = vec![("you", "You"), ("spouse", "Spouse")];

        for (name, display_name) in users {
            sqlx::query(
                "INSERT INTO users (name, display_name) 
                 VALUES ($1, $2) 
                 ON CONFLICT (name) DO NOTHING",
            )
            .bind(name)
            .bind(display_name)
            .execute(self.db.pool())
            .await?;
        }

        println!("✅ Sample users created");
        Ok(())
    }

    /// Create sample ownership relationships
    pub async fn create_sample_ownership(&self) -> Result<()> {
        println!("🤝 Creating sample ownership relationships..."); // Get user IDs
        let you_id: Option<Uuid> = sqlx::query("SELECT id FROM users WHERE name = 'you'")
            .fetch_optional(self.db.pool())
            .await?
            .map(|row| row.get("id"));

        let spouse_id: Option<Uuid> = sqlx::query("SELECT id FROM users WHERE name = 'spouse'")
            .fetch_optional(self.db.pool())
            .await?
            .map(|row| row.get("id"));

        if let (Some(you_id), Some(spouse_id)) = (you_id, spouse_id) {
            // Update some account names for clarity - these names are used to fetch accounts
            // No change needed here as we are updating by current name and setting a new name.
            sqlx::query("UPDATE accounts SET name = 'Joint Checking Account' WHERE name = 'Checking Account'")
                .execute(self.db.pool())
                .await?;

            sqlx::query("UPDATE accounts SET name = 'Your Personal Checking' WHERE name = 'Cash'") // Assuming 'Cash' was the old name for '1003'
                .execute(self.db.pool())
                .await?;

            // Create ownership relationships
            // Account names are used here instead of codes
            let ownership_data = vec![
                // Joint accounts (50/50 ownership)
                ("Joint Checking Account", you_id, 0.5),
                ("Joint Checking Account", spouse_id, 0.5),
                ("Primary Residence", you_id, 0.5),
                ("Primary Residence", spouse_id, 0.5),
                ("Home Mortgage", you_id, 0.5),
                ("Home Mortgage", spouse_id, 0.5),
                // Individual accounts (100% ownership)
                ("Your Personal Checking", you_id, 1.0),
                ("Savings Account", spouse_id, 1.0), // Spouse's Savings (using existing savings account name)
                ("Salary", you_id, 1.0),             // Your Salary
                ("Bonus", spouse_id, 1.0), // Spouse's Bonus (using existing bonus account name)
            ];

            for (account_name, user_id, ownership_percentage) in ownership_data {
                let account_id: Option<Uuid> =
                    sqlx::query("SELECT id FROM accounts WHERE name = $1") // Fetch by name
                        .bind(account_name)
                        .fetch_optional(self.db.pool())
                        .await?
                        .map(|row| row.get("id"));

                if let Some(account_id) = account_id {
                    sqlx::query(
                        "INSERT INTO account_ownership (account_id, user_id, ownership_percentage) 
                         VALUES ($1, $2, $3) 
                         ON CONFLICT (account_id, user_id) DO NOTHING",
                    )
                    .bind(account_id)
                    .bind(user_id)
                    .bind(ownership_percentage)
                    .execute(self.db.pool())
                    .await?;
                }
            }
        }

        println!("✅ Sample ownership relationships created");
        Ok(())
    }

    /// Create sample transactions with journal entries
    pub async fn create_sample_transactions(&self) -> Result<()> {
        println!("💸 Creating sample transactions...");

        use crate::models::{NewJournalEntry, NewTransaction};
        use crate::services::TransactionService;
        use chrono::Utc;
        use rust_decimal::Decimal;
        use std::str::FromStr;

        let transaction_service = TransactionService::new(self.db.pool().clone());

        // Get account IDs by name instead of code
        let mut account_ids = std::collections::HashMap::new();
        // Use the names defined in create_sample_accounts or updated in create_sample_ownership
        let account_names_to_fetch = [
            "Joint Checking Account",
            "Your Personal Checking",
            "Credit Card",
            "Salary",
            "Groceries",
            "Restaurants",
            "Gas",
        ];
        for name in account_names_to_fetch {
            let account_id: Option<uuid::Uuid> =
                sqlx::query("SELECT id FROM accounts WHERE name = $1") // Fetch by name
                    .bind(name)
                    .fetch_optional(self.db.pool())
                    .await?
                    .map(|row| row.get("id"));
            if let Some(id) = account_id {
                account_ids.insert(name, id);
            }
        }

        // Sample transactions - use names as keys for account_ids map
        let transactions = vec![            // 1. Salary payment
            NewTransaction {
                description: "Monthly salary payment".to_string(),
                reference: Some("PAY-2025-01".to_string()),
                transaction_date: Utc::now(),
                created_by: None,
                entries: vec![
                    NewJournalEntry {
                        account_id: account_ids["Joint Checking Account"],
                        amount: Decimal::from_str("3000.00").unwrap(),
                        memo: Some("Salary deposit".to_string()),
                    },
                    NewJournalEntry {
                        account_id: account_ids["Salary"],
                        amount: Decimal::from_str("-3000.00").unwrap(),
                        memo: Some("Monthly salary".to_string()),
                    },
                ],
                import_source: None,
                import_batch_id: None,
                external_reference: None,
            },
            // 2. Grocery purchase
            NewTransaction {
                description: "Weekly groceries".to_string(),
                reference: None,
                transaction_date: Utc::now(),
                created_by: None,
                entries: vec![
                    NewJournalEntry {
                        account_id: account_ids["Groceries"],
                        amount: Decimal::from_str("150.00").unwrap(),
                        memo: Some("Weekly shopping".to_string()),
                    },
                    NewJournalEntry {
                        account_id: account_ids["Credit Card"],
                        amount: Decimal::from_str("-150.00").unwrap(),
                        memo: Some("Grocery payment".to_string()),
                    },                ],
                import_source: None,
                import_batch_id: None,
                external_reference: None,
            },
            // 3. Restaurant meal
            NewTransaction {
                description: "Dinner at restaurant".to_string(),
                reference: None,
                transaction_date: Utc::now(),
                created_by: None,
                entries: vec![
                    NewJournalEntry {
                        account_id: account_ids["Restaurants"],
                        amount: Decimal::from_str("80.00").unwrap(),
                        memo: Some("Family dinner".to_string()),
                    },
                    NewJournalEntry {
                        account_id: account_ids["Your Personal Checking"],
                        amount: Decimal::from_str("-80.00").unwrap(),
                        memo: Some("Restaurant payment".to_string()),
                    },
                ],
                import_source: None,
                import_batch_id: None,
                external_reference: None,
            },
            // 4. Gas purchase
            NewTransaction {
                description: "Gas station fill-up".to_string(),
                reference: None,
                transaction_date: Utc::now(),
                created_by: None,
                entries: vec![
                    NewJournalEntry {
                        account_id: account_ids["Gas"],
                        amount: Decimal::from_str("65.00").unwrap(),
                        memo: Some("Fuel for car".to_string()),
                    },
                    NewJournalEntry {
                        account_id: account_ids["Credit Card"],
                        amount: Decimal::from_str("-65.00").unwrap(),
                        memo: Some("Gas payment".to_string()),
                    },
                ],
                import_source: None,
                import_batch_id: None,
                external_reference: None,
            },
        ]; // Create each transaction
        for transaction in transactions {
            let description = transaction.description.clone();
            match transaction_service.create_transaction(transaction).await {
                Ok(_) => println!("   ✅ Created transaction: {}", description),
                Err(e) => println!("   ❌ Failed to create transaction: {}", e),
            }
        }

        println!("✅ Sample transactions created");
        Ok(())
    }

    /// Create a complete sample dataset (all of the above)
    pub async fn create_full_sample_dataset(&self) -> Result<()> {
        println!("🎯 Creating complete sample dataset...");
        println!("====================================\n");
        self.create_sample_categories().await?;
        self.create_sample_accounts().await?;
        self.create_sample_users().await?;
        self.create_sample_ownership().await?;
        self.create_sample_transactions().await?;
        self.create_sample_transactions().await?;
        println!("\n🎉 Complete sample dataset created successfully!");
        println!("\n📋 What was created:");
        println!("   • Sample categories and subcategories");
        println!("   • Chart of accounts (Assets, Liabilities, Equity, Income, Expenses)");
        println!("   • Sample users (You, Spouse)");
        println!("   • Ownership relationships (joint and individual accounts)");
        println!("   • Sample transactions with journal entries");
        println!("\n🧪 **TESTING COMMANDS** - Try these to explore the system:");
        println!("\n📊 **Account & Balance Commands:**");
        println!(
            "   cargo run -- accounts tree              # Beautiful hierarchical chart of accounts"
        );
        println!("   cargo run -- accounts list              # Flat account list");
        println!(
            "   cargo run -- accounts balance           # All account balances from transactions"
        );
        println!(
            "   cargo run -- accounts ownership \"Joint Checking Account\"    # Show joint account ownership (use name)"
        );
        println!("\n🎭 **Demo & Educational Commands:**");
        println!("   cargo run -- demo double-entry          # Learn double-entry bookkeeping");
        println!("   cargo run -- demo account-types         # Understand debit/credit behavior");
        println!("   cargo run -- demo multi-user            # Multi-user finance examples");
        println!("   cargo run -- demo ownership             # Ownership split scenarios");
        println!("   cargo run -- demo categories            # Category hierarchy examples");
        println!("\n🏗️  **Data Creation Commands:**");
        println!("   cargo run -- demo create-deep-accounts  # Create 4-level account hierarchy");
        println!("   cargo run -- demo create-deep-categories # Create nested category examples");
        println!("\n👥 **User Context Commands:**");
        println!("   cargo run --user you -- accounts balance    # Your perspective only");
        println!("   cargo run --user spouse -- accounts balance # Spouse's perspective only");
        println!("   cargo run --user family -- accounts balance # Combined family view (default)");
        println!("\n🗄️  **Database Commands:**");
        println!("   cargo run -- db status                  # Check database connection");
        println!("   cargo run -- db init                    # Reinitialize database");
        println!("\n🎯 **Quick Test Sequence:**");
        println!("   1. cargo run -- demo create-deep-accounts");
        println!("   2. cargo run -- accounts tree");
        println!("   3. cargo run -- accounts balance");
        println!("   4. cargo run -- demo double-entry");
        println!("   5. cargo run --user you -- accounts balance");

        Ok(())
    }

    /// Create deep category hierarchies to demonstrate unlimited nesting
    pub async fn create_deep_category_hierarchy(&self) -> Result<()> {
        println!("🗂️  Creating deep category hierarchy example...");

        // Create the deep hierarchy: Expense->Home->Deco->Furniture->Sofa
        let hierarchy = vec![
            ("Expense", None),
            ("Home", Some("Expense")),
            ("Deco", Some("Home")),
            ("Furniture", Some("Deco")),
            ("Sofa", Some("Furniture")),
        ];
        for (name, parent_name) in hierarchy {
            let parent_id: Option<Uuid> = if let Some(parent) = parent_name {
                sqlx::query("SELECT id FROM categories WHERE name = $1")
                    .bind(parent)
                    .fetch_optional(self.db.pool())
                    .await?
                    .map(|row| row.get("id"))
            } else {
                None
            };

            sqlx::query(
                "INSERT INTO categories (name, parent_id, color, is_active) 
                 VALUES ($1, $2, $3, $4) 
                 ON CONFLICT (name) DO NOTHING",
            )
            .bind(name)
            .bind(parent_id)
            .bind("#FF6B6B") // Red for expenses
            .bind(true)
            .execute(self.db.pool())
            .await?;

            println!("   ✅ Created: {}", name);
        }

        // Create transportation hierarchy
        let transport_hierarchy = vec![
            ("Transportation", None),
            ("Vehicle Expenses", Some("Transportation")),
            ("Fuel", Some("Vehicle Expenses")),
            ("Gasoline", Some("Fuel")),
            ("Premium Gas", Some("Gasoline")),
        ];
        for (name, parent_name) in transport_hierarchy {
            let parent_id: Option<Uuid> = if let Some(parent) = parent_name {
                sqlx::query("SELECT id FROM categories WHERE name = $1")
                    .bind(parent)
                    .fetch_optional(self.db.pool())
                    .await?
                    .map(|row| row.get("id"))
            } else {
                None
            };

            sqlx::query(
                "INSERT INTO categories (name, parent_id, color, is_active) 
                 VALUES ($1, $2, $3, $4) 
                 ON CONFLICT (name) DO NOTHING",
            )
            .bind(name)
            .bind(parent_id)
            .bind("#4ECDC4") // Teal for transportation
            .bind(true)
            .execute(self.db.pool())
            .await?;
        }

        println!("✅ Deep category hierarchies created!");
        Ok(())
    }

    /// Create deep account hierarchies to demonstrate unlimited nesting
    pub async fn create_deep_account_hierarchy(&self) -> Result<()> {
        println!("🏗️  Creating deep account hierarchy example...");

        let mut tx = self.db.pool().begin().await?;

        // Helper to create or get an account
        async fn get_or_create_account<'a>(
            tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
            name: &str,
            account_type: &str,
            account_subtype: &str,
            is_category: bool,
            parent_id: Option<Uuid>,
        ) -> Result<Uuid> {
            // Check if account exists
            let existing_account: Option<Uuid> = sqlx::query_scalar(
                "SELECT id FROM accounts WHERE name = $1 AND (parent_id = $2 OR (parent_id IS NULL AND $2 IS NULL))",
            )
            .bind(name)
            .bind(parent_id)
            .fetch_optional(&mut **tx)
            .await?;

            if let Some(id) = existing_account {
                return Ok(id);
            }

            // Create account if it doesn't exist
            let account_id = sqlx::query_scalar(
                "INSERT INTO accounts (name, account_type, account_subtype, is_category, parent_id, currency) 
                 VALUES ($1, $2::account_type, $3::account_subtype, $4, $5, $6)
                 ON CONFLICT (name, parent_id) DO UPDATE SET account_type = EXCLUDED.account_type -- Or simply DO NOTHING if updates are not desired
                 RETURNING id"
            )
            .bind(name)
            .bind(account_type)
            .bind(account_subtype)
            .bind(is_category)
            .bind(parent_id)
            .bind("USD") // Default currency
            .fetch_one(&mut **tx)
            .await?;
            Ok(account_id)
        }

        // Level 1: Root Categories (these are also accounts)
        let assets_id =
            get_or_create_account(&mut tx, "Assets", "asset", "category", true, None).await?;
        let liabilities_id =
            get_or_create_account(&mut tx, "Liabilities", "liability", "category", true, None)
                .await?;
        let equity_id =
            get_or_create_account(&mut tx, "Equity", "equity", "category", true, None).await?;
        let income_id =
            get_or_create_account(&mut tx, "Income", "income", "category", true, None).await?;
        let expenses_id =
            get_or_create_account(&mut tx, "Expenses", "expense", "category", true, None).await?;

        // Level 2: Sub-Categories under Assets
        let current_assets_id = get_or_create_account(
            &mut tx,
            "Current Assets",
            "asset",
            "category",
            true,
            Some(assets_id),
        )
        .await?;
        let fixed_assets_id = get_or_create_account(
            &mut tx,
            "Fixed Assets",
            "asset",
            "category",
            true,
            Some(assets_id),
        )
        .await?;
        let investments_cat_id = get_or_create_account(
            &mut tx,
            "Investments Category",
            "asset",
            "category",
            true,
            Some(assets_id),
        )
        .await?;

        // Level 3: Accounts under Current Assets
        let _checking_account_id = get_or_create_account(
            &mut tx,
            "Main Checking",
            "asset",
            "checking",
            false,
            Some(current_assets_id),
        )
        .await?;
        let _savings_account_id = get_or_create_account(
            &mut tx,
            "Emergency Fund",
            "asset",
            "savings",
            false,
            Some(current_assets_id),
        )
        .await?;

        // Level 3: Accounts under Fixed Assets
        let _real_estate_id = get_or_create_account(
            &mut tx,
            "Primary Residence",
            "asset",
            "real_estate",
            false,
            Some(fixed_assets_id),
        )
        .await?;

        // Level 3: Accounts under Investments Category
        let _brokerage_id = get_or_create_account(
            &mut tx,
            "Brokerage Account",
            "asset",
            "investment_account",
            false,
            Some(investments_cat_id),
        )
        .await?;

        // Level 2: Sub-Categories under Liabilities
        let current_liabilities_id = get_or_create_account(
            &mut tx,
            "Current Liabilities",
            "liability",
            "category",
            true,
            Some(liabilities_id),
        )
        .await?;
        let long_term_liabilities_id = get_or_create_account(
            &mut tx,
            "Long-Term Liabilities",
            "liability",
            "category",
            true,
            Some(liabilities_id),
        )
        .await?;

        // Level 3: Accounts under Current Liabilities
        let _credit_card_id = get_or_create_account(
            &mut tx,
            "Visa Credit Card",
            "liability",
            "credit_card",
            false,
            Some(current_liabilities_id),
        )
        .await?;

        // Level 3: Accounts under Long-Term Liabilities
        let _mortgage_id = get_or_create_account(
            &mut tx,
            "Home Mortgage",
            "liability",
            "mortgage",
            false,
            Some(long_term_liabilities_id),
        )
        .await?;

        // Level 2: Sub-Categories under Income
        let earned_income_id = get_or_create_account(
            &mut tx,
            "Earned Income",
            "income",
            "category",
            true,
            Some(income_id),
        )
        .await?;
        let investment_income_id = get_or_create_account(
            &mut tx,
            "Investment Income",
            "income",
            "category",
            true,
            Some(income_id),
        )
        .await?;

        // Level 3: Accounts under Earned Income
        let _salary_id = get_or_create_account(
            &mut tx,
            "Salary",
            "income",
            "salary",
            false,
            Some(earned_income_id),
        )
        .await?;

        // Level 3: Accounts under Investment Income
        let _dividends_id = get_or_create_account(
            &mut tx,
            "Dividends",
            "income",
            "dividend",
            false,
            Some(investment_income_id),
        )
        .await?;

        // Level 2: Sub-Categories under Expenses
        let housing_expenses_id = get_or_create_account(
            &mut tx,
            "Housing",
            "expense",
            "category",
            true,
            Some(expenses_id),
        )
        .await?;
        let food_expenses_id = get_or_create_account(
            &mut tx,
            "Food",
            "expense",
            "category",
            true,
            Some(expenses_id),
        )
        .await?;
        let transport_expenses_id = get_or_create_account(
            &mut tx,
            "Transportation",
            "expense",
            "category",
            true,
            Some(expenses_id),
        )
        .await?;

        // Level 3: Accounts under Housing
        let _rent_id = get_or_create_account(
            &mut tx,
            "Rent/Mortgage Payment",
            "expense",
            "housing",
            false,
            Some(housing_expenses_id),
        )
        .await?;
        let _utilities_id = get_or_create_account(
            &mut tx,
            "Utilities",
            "expense",
            "utilities",
            false,
            Some(housing_expenses_id),
        )
        .await?;

        // Level 3: Accounts under Food
        let _groceries_id = get_or_create_account(
            &mut tx,
            "Groceries",
            "expense",
            "food",
            false,
            Some(food_expenses_id),
        )
        .await?;
        let _restaurants_id = get_or_create_account(
            &mut tx,
            "Restaurants",
            "expense",
            "food",
            false,
            Some(food_expenses_id),
        )
        .await?;

        // Level 3: Accounts under Transportation
        let _fuel_id = get_or_create_account(
            &mut tx,
            "Fuel",
            "expense",
            "transportation",
            false,
            Some(transport_expenses_id),
        )
        .await?;
        let _public_transport_id = get_or_create_account(
            &mut tx,
            "Public Transport",
            "expense",
            "transportation",
            false,
            Some(transport_expenses_id),
        )
        .await?;

        // Level 2: Sub-Categories under Equity
        // (Example: Owner's Equity, Retained Earnings - often simpler structure)
        let _owners_equity_id = get_or_create_account(
            &mut tx,
            "Owner's Equity",
            "equity",
            "owner_equity",
            false,
            Some(equity_id),
        )
        .await?; // Typically not a category itself
        let _retained_earnings_id = get_or_create_account(
            &mut tx,
            "Retained Earnings",
            "equity",
            "retained_earnings",
            false,
            Some(equity_id),
        )
        .await?; // Typically not a category itself

        tx.commit().await?;
        println!("✅ Deep account hierarchy created successfully.");
        Ok(())
    }
    /// Create sample price data for investment accounts
    pub async fn create_sample_price_data(&self) -> Result<()> {
        use chrono::{Duration, Utc};
        use rust_decimal::Decimal;
        use rust_decimal::prelude::FromPrimitive;
        use std::str::FromStr;

        println!("📈 Creating sample price data...");

        // Sample symbols with realistic prices in EUR
        let sample_prices = vec![
            ("AAPL", "150.00", "Apple Inc."),
            ("GOOGL", "2800.00", "Alphabet Inc."),
            ("MSFT", "350.00", "Microsoft Corp."),
            ("TSLA", "800.00", "Tesla Inc."),
            ("SPY", "450.00", "SPDR S&P 500 ETF"),
            ("QQQ", "380.00", "Invesco QQQ Trust"),
            ("VTI", "220.00", "Vanguard Total Stock Market ETF"),
            ("BTC", "45000.00", "Bitcoin"),
            ("ETH", "3000.00", "Ethereum"),
        ];
        let today = Utc::now().naive_utc().date();
        let symbols_count = sample_prices.len();

        for (symbol, price_str, _description) in &sample_prices {
            let base_price = Decimal::from_str(price_str).map_err(|e| {
                crate::error::CoreError::Generic(format!("Invalid price format: {}", e))
            })?;

            // Create price history for the last 30 days with some variation
            for days_ago in (0..30).step_by(3) {
                let price_date = today - Duration::days(days_ago);

                // Add some realistic price variation (±1.5%)
                let variation = (days_ago as f64 * 0.0005) - 0.015; // Small daily variation
                let price_multiplier = Decimal::from_f64(1.0 + variation).unwrap_or(Decimal::ONE);
                let price = base_price * price_multiplier;

                sqlx::query(
                    "INSERT INTO price_history (symbol, price, price_date, currency, source) 
                     VALUES ($1, $2, $3, $4, $5) 
                     ON CONFLICT (symbol, price_date) DO UPDATE SET 
                     price = $2, currency = $4, source = $5",
                )
                .bind(symbol)
                .bind(price)
                .bind(price_date)
                .bind("EUR")
                .bind("sample_data")
                .execute(self.db.pool())
                .await?;
            }
        }

        println!("✅ Sample price data created for {} symbols", symbols_count);
        println!("   Data covers the last 30 days with 3-day intervals");
        Ok(())
    }

    /// Remove all sample data (useful for cleanup)
    pub async fn clear_sample_data(&self) -> Result<()> {
        println!("🧹 Clearing all sample data...");

        // Order matters due to foreign key constraints
        sqlx::query("DELETE FROM account_ownership")
            .execute(self.db.pool())
            .await?;
        sqlx::query("DELETE FROM users WHERE name IN ('you', 'spouse')")
            .execute(self.db.pool())
            .await?;
        sqlx::query("DELETE FROM accounts")
            .execute(self.db.pool())
            .await?;
        sqlx::query("DELETE FROM categories")
            .execute(self.db.pool())
            .await?;

        println!("✅ Sample data cleared");
        Ok(())
    }
}
