//use crate::models::{Account, AccountType, Transaction, JournalEntry, NewTransaction, NewJournalEntry, TransactionWithEntries, User, AccountOwnership, AccountOwnershipView}; Business logic and services for financial operations

use crate::error::Result;
use crate::models::{
    Account, AccountOwnership, AccountOwnershipWithUser, AccountType, AccountWithOwnership,
    AccountWithOwnershipAndUsers, JournalEntry, NewAccount, NewJournalEntry, NewTransaction,
    Transaction, TransactionWithEntries, User, PriceHistory, NewPriceHistory, AccountWithMarketValue,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

pub struct TransactionService {
    pool: PgPool,
}

impl TransactionService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new transaction with journal entries
    /// Validates that the transaction balances before inserting
    pub async fn create_transaction(
        &self,
        new_transaction: NewTransaction,
    ) -> Result<TransactionWithEntries> {
        if !new_transaction.is_balanced() {
            return Err(crate::error::CoreError::UnbalancedTransaction {
                expected: Decimal::ZERO,
                actual: new_transaction.entries.iter().map(|e| e.amount).sum(),
            });
        }

        let mut tx = self.pool.begin().await?; // Insert transaction header
        let transaction_id = Uuid::new_v4();
        let transaction = sqlx::query_as::<_, Transaction>(
            r#"
            INSERT INTO transactions (id, description, reference, transaction_date, created_by)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, description, reference, transaction_date, created_by, created_at
            "#,
        )
        .bind(transaction_id)
        .bind(&new_transaction.description)
        .bind(&new_transaction.reference)
        .bind(new_transaction.transaction_date)
        .bind(new_transaction.created_by)
        .fetch_one(&mut *tx)
        .await?; // Insert journal entries
        let mut entries = Vec::new();
        for entry in new_transaction.entries {
            let journal_entry = sqlx::query_as::<_, JournalEntry>(
                r#"
                INSERT INTO journal_entries (transaction_id, account_id, amount, memo)
                VALUES ($1, $2, $3, $4)
                RETURNING id, transaction_id, account_id, amount, memo, created_at
                "#,
            )
            .bind(transaction_id)
            .bind(entry.account_id)
            .bind(entry.amount)
            .bind(&entry.memo)
            .fetch_one(&mut *tx)
            .await?;
            entries.push(journal_entry);
        }

        tx.commit().await?;

        Ok(TransactionWithEntries {
            transaction,
            entries,
        })
    }
    /// Get a transaction with all its journal entries
    pub async fn get_transaction(
        &self,
        transaction_id: Uuid,
    ) -> Result<Option<TransactionWithEntries>> {
        let transaction = sqlx::query_as::<_, Transaction>(
            "SELECT id, description, reference, transaction_date, created_by, created_at FROM transactions WHERE id = $1",
        )
        .bind(transaction_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(transaction) = transaction else {
            return Ok(None);
        };

        let entries = sqlx::query_as::<_, JournalEntry>(
            "SELECT id, transaction_id, account_id, amount, memo, created_at FROM journal_entries WHERE transaction_id = $1 ORDER BY created_at",
        )
        .bind(transaction_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(Some(TransactionWithEntries {
            transaction,
            entries,
        }))
    }
    /// Helper: Create a simple two-account transaction (most common case)
    pub fn create_simple_transaction(
        description: String,
        debit_account_id: Uuid,
        credit_account_id: Uuid,
        amount: Decimal,
        transaction_date: DateTime<Utc>,
        reference: Option<String>,
        created_by: Option<Uuid>,
    ) -> NewTransaction {
        NewTransaction {
            description,
            reference,
            transaction_date,
            created_by,
            entries: vec![
                NewJournalEntry {
                    account_id: debit_account_id,
                    amount,
                    memo: None,
                },
                NewJournalEntry {
                    account_id: credit_account_id,
                    amount: -amount,
                    memo: None,
                },
            ],
        }
    }
}

pub struct AccountService {
    pool: PgPool,
}

impl AccountService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    /// Get all accounts
    pub async fn get_all_accounts(&self) -> Result<Vec<Account>> {
        let accounts = sqlx::query_as::<_, Account>(
            r#"
            SELECT 
                id, code, name, 
                account_type, account_subtype,
                parent_id, symbol, quantity, average_cost, address, 
                purchase_date, purchase_price, currency, is_active, 
                notes, created_at, updated_at
            FROM accounts 
            WHERE is_active = true 
            ORDER BY code
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(accounts)
    }
    /// Get accounts by type
    pub async fn get_accounts_by_type(&self, account_type: AccountType) -> Result<Vec<Account>> {
        let accounts = sqlx::query_as::<_, Account>(
            r#"
            SELECT 
                id, code, name, 
                account_type, account_subtype,
                parent_id, symbol, quantity, average_cost, address, 
                purchase_date, purchase_price, currency, is_active, 
                notes, created_at, updated_at
            FROM accounts 
            WHERE account_type = $1 AND is_active = true 
            ORDER BY code
            "#,
        )
        .bind(account_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(accounts)
    }
    /// Get account by code
    pub async fn get_account_by_code(&self, code: &str) -> Result<Option<Account>> {
        let account = sqlx::query_as::<_, Account>(
            r#"
            SELECT 
                id, code, name, 
                account_type, account_subtype,
                parent_id, symbol, quantity, average_cost, address, 
                purchase_date, purchase_price, currency, is_active, 
                notes, created_at, updated_at
            FROM accounts 
            WHERE code = $1
            "#,
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await?;
        Ok(account)
    }

    /// Get account by ID
    pub async fn get_account(&self, account_id: Uuid) -> Result<Option<Account>> {
        let account = sqlx::query_as::<_, Account>(
            r#"
            SELECT 
                id, code, name, 
                account_type, account_subtype,
                parent_id, symbol, quantity, average_cost, address, 
                purchase_date, purchase_price, currency, is_active, 
                notes, created_at, updated_at
            FROM accounts 
            WHERE id = $1
            "#,
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(account)
    }

    /// Get account with ownership information
    pub async fn get_account_with_ownership(
        &self,
        account_id: Uuid,
    ) -> Result<Option<AccountWithOwnership>> {
        // Get the account first
        let account = match self.get_account(account_id).await? {
            Some(account) => account,
            None => return Ok(None),
        }; // Get ownership information
        let ownerships = sqlx::query_as::<_, AccountOwnership>(
            r#"
            SELECT id, user_id, account_id, ownership_percentage, created_at
            FROM account_ownership
            WHERE account_id = $1
            ORDER BY ownership_percentage DESC
            "#,
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(Some(AccountWithOwnership {
            account,
            ownership: ownerships,
            user_balance: None,
            user_percentage: None,
        }))
    }

    /// Get account with ownership information including user details - avoids multiple database round trips
    pub async fn get_account_with_ownership_and_users(
        &self,
        account_id: Uuid,
    ) -> Result<Option<AccountWithOwnershipAndUsers>> {
        // Get the account first
        let account = match self.get_account(account_id).await? {
            Some(account) => account,
            None => return Ok(None),
        };

        // Get ownership information with user details in a single query
        let ownerships = sqlx::query_as::<_, AccountOwnershipWithUser>(
            r#"
            SELECT 
                ao.id, 
                ao.user_id, 
                ao.account_id, 
                ao.ownership_percentage, 
                ao.created_at,
                u.name as user_name,
                u.display_name as user_display_name,
                u.is_active as user_is_active
            FROM account_ownership ao
            JOIN users u ON ao.user_id = u.id
            WHERE ao.account_id = $1
            ORDER BY ao.ownership_percentage DESC
            "#,
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(Some(AccountWithOwnershipAndUsers {
            account,
            ownership: ownerships,
            user_balance: None,
            user_percentage: None,
        }))
    }
    /// Create a new account
    pub async fn create_account(&self, new_account: NewAccount) -> Result<Account> {
        let account = sqlx::query_as::<_, Account>(
            r#"
            INSERT INTO accounts (
                code, name, account_type, account_subtype, parent_id,
                symbol, quantity, average_cost, address, purchase_date, 
                purchase_price, currency, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING 
                id, code, name, account_type, account_subtype, parent_id,
                symbol, quantity, average_cost, address, purchase_date,
                purchase_price, currency, is_active, notes, created_at, updated_at
            "#,
        )
        .bind(&new_account.code)
        .bind(&new_account.name)
        .bind(&new_account.account_type)
        .bind(&new_account.account_subtype)
        .bind(new_account.parent_id)
        .bind(&new_account.symbol)
        .bind(new_account.quantity)
        .bind(new_account.average_cost)
        .bind(&new_account.address)
        .bind(new_account.purchase_date)
        .bind(new_account.purchase_price)
        .bind(&new_account.currency)
        .bind(&new_account.notes)
        .fetch_one(&self.pool)
        .await?;

        Ok(account)
    }

    /// Generate the next available account code for a given account type
    pub async fn generate_account_code(&self, account_type: AccountType) -> Result<String> {
        // Account code ranges by type (following standard chart of accounts)
        let (prefix, start_range) = match account_type {
            AccountType::Asset => ("1", 1000),
            AccountType::Liability => ("2", 2000),
            AccountType::Equity => ("3", 3000),
            AccountType::Income => ("4", 4000),
            AccountType::Expense => ("5", 5000),
        };

        // Find the highest existing code in this range
        let max_code: Option<String> = sqlx::query_scalar(
            "SELECT code FROM accounts WHERE code LIKE $1 ORDER BY code DESC LIMIT 1",
        )
        .bind(format!("{}%", prefix))
        .fetch_optional(&self.pool)
        .await?;

        let next_number = if let Some(code) = max_code {
            // Parse the numeric part and increment
            if let Ok(num) = code.parse::<i32>() {
                num + 1
            } else {
                start_range
            }
        } else {
            start_range
        };

        Ok(next_number.to_string())
    }

    /// Create a new account with ownership in a single transaction
    /// This ensures that if ownership setup fails, the account creation is rolled back
    pub async fn create_account_with_ownership(
        &self,
        new_account: NewAccount,
        ownership: Vec<(Uuid, Decimal)>,
    ) -> Result<Account> {
        // Validate that percentages sum to 100 or less
        let total: Decimal = ownership.iter().map(|(_, pct)| pct).sum();
        if total > Decimal::from(1) {
            return Err(crate::error::CoreError::InvalidInput(
                "Total ownership percentage cannot exceed 100%".to_string(),
            ));
        }

        let mut tx = self.pool.begin().await?;

        // Create the account
        let account = sqlx::query_as::<_, Account>(
            r#"
            INSERT INTO accounts (
                code, name, account_type, account_subtype, parent_id,
                symbol, quantity, average_cost, address, purchase_date, 
                purchase_price, currency, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING 
                id, code, name, account_type, account_subtype, parent_id,
                symbol, quantity, average_cost, address, purchase_date,
                purchase_price, currency, is_active, notes, created_at, updated_at
            "#,
        )
        .bind(&new_account.code)
        .bind(&new_account.name)
        .bind(&new_account.account_type)
        .bind(&new_account.account_subtype)
        .bind(new_account.parent_id)
        .bind(&new_account.symbol)
        .bind(new_account.quantity)
        .bind(new_account.average_cost)
        .bind(&new_account.address)
        .bind(new_account.purchase_date)
        .bind(new_account.purchase_price)
        .bind(&new_account.currency)
        .bind(&new_account.notes)
        .fetch_one(&mut *tx)
        .await?;

        // Set up ownership if provided
        if !ownership.is_empty() {
            for (user_id, percentage) in ownership {
                sqlx::query(
                    "INSERT INTO account_ownership (user_id, account_id, ownership_percentage) VALUES ($1, $2, $3)"
                )
                .bind(user_id)
                .bind(account.id)
                .bind(percentage)
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
        Ok(account)
    }
}

pub struct UserService {
    pool: PgPool,
}

impl UserService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get all users
    pub async fn get_all_users(&self) -> Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>(
            "SELECT id, name, display_name, is_active, created_at FROM users WHERE is_active = true ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    /// Get user by name
    pub async fn get_user_by_name(&self, name: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, name, display_name, is_active, created_at FROM users WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Create a new user
    pub async fn create_user(&self, name: String, display_name: String) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (name, display_name)
            VALUES ($1, $2)
            RETURNING id, name, display_name, is_active, created_at
            "#,
        )
        .bind(&name)
        .bind(&display_name)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }
}

pub struct OwnershipService {
    pool: PgPool,
}

impl OwnershipService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    /// Get ownership for a specific account
    pub async fn get_account_ownership(&self, account_id: Uuid) -> Result<Vec<AccountOwnership>> {
        let ownership = sqlx::query_as::<_, AccountOwnership>(
            r#"
            SELECT id, user_id, account_id, ownership_percentage, created_at
            FROM account_ownership
            WHERE account_id = $1
            ORDER BY ownership_percentage DESC
            "#,
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(ownership)
    }
    /// Get all accounts owned by a user (with ownership percentage)
    pub async fn get_user_accounts(&self, user_id: Uuid) -> Result<Vec<AccountWithOwnership>> {
        // For now, we'll use a simpler approach that doesn't rely on compile-time query validation
        let ownership_records = sqlx::query_as::<_, AccountOwnership>(
            "SELECT id, user_id, account_id, ownership_percentage, created_at FROM account_ownership WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for ownership in ownership_records {
            // Get the account details
            if let Ok(Some(account)) = sqlx::query_as::<_, Account>(
                r#"
                SELECT 
                    id, code, name, account_type, account_subtype,
                    parent_id, symbol, quantity, average_cost, address,
                    purchase_date, purchase_price, currency, is_active,
                    notes, created_at, updated_at
                FROM accounts WHERE id = $1 AND is_active = true
                "#,
            )
            .bind(ownership.account_id)
            .fetch_optional(&self.pool)
            .await
            {
                // Get full ownership info for this account
                let all_ownership = self.get_account_ownership(ownership.account_id).await?;

                // Calculate user's balance
                let total_balance = account.calculate_balance(&self.pool).await?;
                let user_balance =
                    total_balance * ownership.ownership_percentage / Decimal::from(100);

                result.push(AccountWithOwnership {
                    account,
                    ownership: all_ownership,
                    user_balance: Some(user_balance),
                    user_percentage: Some(ownership.ownership_percentage),
                });
            }
        }

        Ok(result)
    }

    /// Set ownership for an account (replaces existing ownership)
    pub async fn set_account_ownership(
        &self,
        account_id: Uuid,
        ownership: Vec<(Uuid, Decimal)>,
    ) -> Result<()> {
        // Validate that percentages sum to 100 or less
        let total: Decimal = ownership.iter().map(|(_, pct)| pct).sum();
        if total > Decimal::from(100) {
            return Err(crate::error::CoreError::InvalidInput(
                "Total ownership percentage cannot exceed 100%".to_string(),
            ));
        }

        let mut tx = self.pool.begin().await?;

        // Delete existing ownership
        sqlx::query("DELETE FROM account_ownership WHERE account_id = $1")
            .bind(account_id)
            .execute(&mut *tx)
            .await?;

        // Insert new ownership
        for (user_id, percentage) in ownership {
            sqlx::query(
                "INSERT INTO account_ownership (user_id, account_id, ownership_percentage) VALUES ($1, $2, $3)"
            )
            .bind(user_id)
            .bind(account_id)
            .bind(percentage)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}

pub struct PriceHistoryService {
    pool: PgPool,
}

impl PriceHistoryService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Add or update a price entry for a symbol on a specific date
    pub async fn add_price(
        &self,
        new_price: NewPriceHistory,
    ) -> Result<PriceHistory> {
        let price = sqlx::query_as::<_, PriceHistory>(
            r#"
            INSERT INTO price_history (symbol, price, price_date, currency, source)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (symbol, price_date) 
            DO UPDATE SET 
                price = EXCLUDED.price,
                currency = EXCLUDED.currency,
                source = EXCLUDED.source,
                created_at = NOW()
            RETURNING id, symbol, price, price_date, currency, source, created_at
            "#,
        )
        .bind(&new_price.symbol.to_uppercase())
        .bind(new_price.price)
        .bind(new_price.price_date)
        .bind(&new_price.currency)
        .bind(&new_price.source)
        .fetch_one(&self.pool)
        .await?;

        Ok(price)
    }

    /// Get the latest price for a symbol
    pub async fn get_latest_price(&self, symbol: &str) -> Result<Option<PriceHistory>> {
        let price = sqlx::query_as::<_, PriceHistory>(
            r#"
            SELECT id, symbol, price, price_date, currency, source, created_at
            FROM price_history
            WHERE symbol = $1
            ORDER BY price_date DESC
            LIMIT 1
            "#,
        )
        .bind(symbol.to_uppercase())
        .fetch_optional(&self.pool)
        .await?;

        Ok(price)
    }

    /// Get price history for a symbol over a date range
    pub async fn get_price_history(
        &self,
        symbol: &str,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
    ) -> Result<Vec<PriceHistory>> {
        let mut query = "SELECT id, symbol, price, price_date, currency, source, created_at FROM price_history WHERE symbol = $1".to_string();
        let mut params = vec![symbol.to_uppercase()];
        let mut param_count = 1;

        if let Some(start) = start_date {
            param_count += 1;
            query.push_str(&format!(" AND price_date >= ${}", param_count));
            params.push(start.to_string());
        }

        if let Some(end) = end_date {
            param_count += 1;
            query.push_str(&format!(" AND price_date <= ${}", param_count));
            params.push(end.to_string());
        }

        query.push_str(" ORDER BY price_date ASC");

        let mut query_builder = sqlx::query_as::<_, PriceHistory>(&query);
        for param in params {
            query_builder = query_builder.bind(param);
        }

        let prices = query_builder.fetch_all(&self.pool).await?;
        Ok(prices)
    }

    /// Get all symbols that have price history
    pub async fn get_tracked_symbols(&self) -> Result<Vec<String>> {
        let symbols: Vec<String> = sqlx::query_scalar(
            "SELECT DISTINCT symbol FROM price_history ORDER BY symbol",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(symbols)
    }

    /// Calculate market value for an account with investments
    pub async fn get_account_with_market_value(
        &self,
        account: Account,
    ) -> Result<AccountWithMarketValue> {
        // Calculate book value from journal entries
        let book_value = account.calculate_balance(&self.pool).await
            .map_err(|e| crate::error::CoreError::Database(e))?;

        let mut market_value = None;
        let mut unrealized_gain_loss = None;
        let mut latest_price = None;

        // For investment accounts with symbol and quantity, calculate market value
        if let (Some(symbol), Some(quantity)) = (&account.symbol, account.quantity) {
            if let Ok(Some(price)) = self.get_latest_price(symbol).await {
                let current_market_value = quantity * price.price;
                market_value = Some(current_market_value);
                unrealized_gain_loss = Some(current_market_value - book_value);
                latest_price = Some(price);
            }
        }

        Ok(AccountWithMarketValue {
            account,
            book_value,
            market_value,
            unrealized_gain_loss,
            latest_price,
        })
    }
}
