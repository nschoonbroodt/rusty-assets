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
                 ON CONFLICT (name) DO NOTHING"
            )
            .bind(Uuid::new_v4())
            .bind(name)
            .bind(color)
            .bind(true)
            .execute(self.db.pool())
            .await?;
        }

        // Create subcategories for Food & Dining
        let food_category_id: Option<Uuid> = sqlx::query("SELECT id FROM categories WHERE name = 'Food & Dining'")
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
                     ON CONFLICT (name) DO NOTHING"
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
            ("1001", "Checking Account", "asset", "checking"),
            ("1002", "Savings Account", "asset", "savings"), 
            ("1003", "Cash", "asset", "cash"),
            ("1100", "Brokerage Account", "asset", "investment_account"),
            ("1101", "Apple Inc. (AAPL)", "asset", "stocks"),
            ("1102", "S&P 500 ETF (SPY)", "asset", "etf"),
            ("1103", "Bitcoin", "asset", "crypto"),
            ("1200", "Primary Residence", "asset", "real_estate"),
            ("1201", "Rental Property", "asset", "real_estate"),
        ];

        // Liabilities (2000-2999)
        let liability_accounts = vec![
            ("2001", "Credit Card", "liability", "credit_card"),
            ("2002", "Home Mortgage", "liability", "mortgage"),
            ("2003", "Car Loan", "liability", "loan"),
        ];

        // Equity (3000-3999)
        let equity_accounts = vec![
            ("3001", "Opening Balance Equity", "equity", "opening_balance"),
            ("3002", "Retained Earnings", "equity", "retained_earnings"),
        ];

        // Income (4000-4999)
        let income_accounts = vec![
            ("4001", "Salary", "income", "salary"),
            ("4002", "Bonus", "income", "bonus"),
            ("4003", "Investment Income", "income", "investment"),
            ("4004", "Rental Income", "income", "rental"),
        ];

        // Expenses (5000-5999)
        let expense_accounts = vec![
            ("5001", "Groceries", "expense", "food"),
            ("5002", "Restaurants", "expense", "food"),
            ("5003", "Gas", "expense", "transportation"),
            ("5004", "Car Maintenance", "expense", "transportation"),
            ("5005", "Rent", "expense", "housing"),
            ("5006", "Utilities", "expense", "housing"),
            ("5007", "Internet", "expense", "housing"),
            ("5008", "Phone", "expense", "communication"),
            ("5009", "Entertainment", "expense", "entertainment"),
            ("5010", "Clothing", "expense", "personal"),
        ];

        let all_accounts = [
            asset_accounts,
            liability_accounts, 
            equity_accounts,
            income_accounts,
            expense_accounts,
        ].concat();        for (code, name, account_type, account_subtype) in all_accounts {
            sqlx::query(
                "INSERT INTO accounts (code, name, account_type, account_subtype) 
                 VALUES ($1, $2, $3::account_type, $4::account_subtype) 
                 ON CONFLICT (code) DO NOTHING"
            )
            .bind(code)
            .bind(name)
            .bind(account_type)
            .bind(account_subtype)
            .execute(self.db.pool())
            .await?;
        }

        println!("✅ Sample chart of accounts created");
        Ok(())
    }    /// Create sample users
    pub async fn create_sample_users(&self) -> Result<()> {
        println!("👥 Creating sample users...");

        let users = vec![
            ("you", "You"),
            ("spouse", "Spouse"),
        ];

        for (name, display_name) in users {
            sqlx::query(
                "INSERT INTO users (name, display_name) 
                 VALUES ($1, $2) 
                 ON CONFLICT (name) DO NOTHING"
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
        println!("🤝 Creating sample ownership relationships...");        // Get user IDs
        let you_id: Option<Uuid> = sqlx::query("SELECT id FROM users WHERE name = 'you'")
            .fetch_optional(self.db.pool())
            .await?
            .map(|row| row.get("id"));

        let spouse_id: Option<Uuid> = sqlx::query("SELECT id FROM users WHERE name = 'spouse'")
            .fetch_optional(self.db.pool())
            .await?
            .map(|row| row.get("id"));

        if let (Some(you_id), Some(spouse_id)) = (you_id, spouse_id) {
            // Update some account names for clarity
            sqlx::query("UPDATE accounts SET name = 'Joint Checking Account' WHERE code = '1001'")
                .execute(self.db.pool())
                .await?;

            sqlx::query("UPDATE accounts SET name = 'Your Personal Checking' WHERE code = '1003'")
                .execute(self.db.pool())
                .await?;

            // Create ownership relationships
            let ownership_data = vec![
                // Joint accounts (50/50 ownership)
                ("1001", you_id, 0.5), // Joint Checking - You
                ("1001", spouse_id, 0.5), // Joint Checking - Spouse
                ("1200", you_id, 0.5), // Primary Residence - You
                ("1200", spouse_id, 0.5), // Primary Residence - Spouse  
                ("2002", you_id, 0.5), // Home Mortgage - You
                ("2002", spouse_id, 0.5), // Home Mortgage - Spouse
                
                // Individual accounts (100% ownership)
                ("1003", you_id, 1.0), // Your Personal Checking
                ("1002", spouse_id, 1.0), // Spouse's Savings (using existing savings account)
                ("4001", you_id, 1.0), // Your Salary
                ("4002", spouse_id, 1.0), // Spouse's Bonus (using existing bonus account)
            ];

            for (account_code, user_id, ownership_percentage) in ownership_data {
                let account_id: Option<Uuid> = sqlx::query("SELECT id FROM accounts WHERE code = $1")
                    .bind(account_code)
                    .fetch_optional(self.db.pool())
                    .await?
                    .map(|row| row.get("id"));

                if let Some(account_id) = account_id {
                    sqlx::query(
                        "INSERT INTO account_ownership (account_id, user_id, ownership_percentage) 
                         VALUES ($1, $2, $3) 
                         ON CONFLICT (account_id, user_id) DO NOTHING"
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

    /// Create a complete sample dataset (all of the above)
    pub async fn create_full_sample_dataset(&self) -> Result<()> {
        println!("🎯 Creating complete sample dataset...");
        println!("====================================\n");

        self.create_sample_categories().await?;
        self.create_sample_accounts().await?;
        self.create_sample_users().await?;
        self.create_sample_ownership().await?;

        println!("\n🎉 Complete sample dataset created successfully!");
        println!("\n📋 What was created:");
        println!("   • Sample categories and subcategories");
        println!("   • Chart of accounts (Assets, Liabilities, Equity, Income, Expenses)");
        println!("   • Sample users (You, Spouse)");
        println!("   • Ownership relationships (joint and individual accounts)");
        println!("\n💡 Try these commands:");
        println!("   cargo run --bin assets-cli -- multi-user");
        println!("   cargo run --bin assets-cli -- ownership");
        println!("   cargo run --bin assets-cli -- categories");

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
        ];        for (name, parent_name) in hierarchy {
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
                 ON CONFLICT (name) DO NOTHING"
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
        ];        for (name, parent_name) in transport_hierarchy {
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
                 ON CONFLICT (name) DO NOTHING"
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

    /// Remove all sample data (useful for cleanup)
    pub async fn clear_sample_data(&self) -> Result<()> {
        println!("🧹 Clearing all sample data...");

        // Order matters due to foreign key constraints
        sqlx::query("DELETE FROM account_ownership").execute(self.db.pool()).await?;
        sqlx::query("DELETE FROM users WHERE name IN ('you', 'spouse')").execute(self.db.pool()).await?;
        sqlx::query("DELETE FROM accounts").execute(self.db.pool()).await?;
        sqlx::query("DELETE FROM categories").execute(self.db.pool()).await?;

        println!("✅ Sample data cleared");
        Ok(())
    }
}
