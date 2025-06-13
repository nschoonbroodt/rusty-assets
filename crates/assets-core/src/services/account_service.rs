use crate::error::Result;
use crate::models::{
    Account, AccountOwnership, AccountOwnershipWithUser, AccountType, AccountWithOwnership,
    AccountWithOwnershipAndUsers, NewAccount,
};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

pub struct AccountService {
    pool: PgPool,
}

impl AccountService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }    /// Get all accounts
    pub async fn get_all_accounts(&self) -> Result<Vec<Account>> {
        let accounts = sqlx::query_as::<_, Account>(
            r#"
            SELECT 
                id, name, full_path,
                account_type, account_subtype,
                parent_id, symbol, quantity, average_cost, address, 
                purchase_date, purchase_price, currency, is_active, 
                notes, created_at, updated_at
            FROM accounts 
            WHERE is_active = true 
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(accounts)
    }    /// Get accounts by type
    pub async fn get_accounts_by_type(&self, account_type: AccountType) -> Result<Vec<Account>> {
        let accounts = sqlx::query_as::<_, Account>(
            r#"
            SELECT 
                id, name, full_path,
                account_type, account_subtype,
                parent_id, symbol, quantity, average_cost, address, 
                purchase_date, purchase_price, currency, is_active, 
                notes, created_at, updated_at
            FROM accounts 
            WHERE account_type = $1 AND is_active = true 
            ORDER BY name
            "#,
        )
        .bind(account_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(accounts)
    }    /// Get account by ID
    pub async fn get_account(&self, account_id: Uuid) -> Result<Option<Account>> {
        let account = sqlx::query_as::<_, Account>(
            r#"
            SELECT 
                id, name, full_path,
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
    }    /// Create a new account with default ownership (100% to first user)
    pub async fn create_account(&self, new_account: NewAccount) -> Result<Account> {
        // Get the first user as default owner
        let user_service = crate::services::UserService::new(self.pool.clone());
        let default_user = user_service.get_first_user().await?;
        
        match default_user {
            Some(user) => {
                // Create account with default 100% ownership to first user
                let ownership = vec![(user.id, Decimal::from(1))]; // 100%
                self.create_account_with_ownership(new_account, ownership).await
            }
            None => {
                // No users exist - create account without ownership for now
                // This should be rare and might indicate a setup issue
                self.create_account_without_ownership(new_account).await
            }
        }
    }

    /// Create a new account without ownership (internal method)
    async fn create_account_without_ownership(&self, new_account: NewAccount) -> Result<Account> {
        let account = sqlx::query_as::<_, Account>(
            r#"
            INSERT INTO accounts (
                name, account_type, account_subtype, parent_id,
                symbol, quantity, average_cost, address, purchase_date, 
                purchase_price, currency, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING 
                id, name, full_path, account_type, account_subtype, parent_id,
                symbol, quantity, average_cost, address, purchase_date,
                purchase_price, currency, is_active, notes, created_at, updated_at
            "#,
        )
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

        let mut tx = self.pool.begin().await?;        // Create the account
        let account = sqlx::query_as::<_, Account>(
            r#"
            INSERT INTO accounts (
                name, account_type, account_subtype, parent_id,
                symbol, quantity, average_cost, address, purchase_date, 
                purchase_price, currency, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING 
                id, name, full_path, account_type, account_subtype, parent_id,
                symbol, quantity, average_cost, address, purchase_date,
                purchase_price, currency, is_active, notes, created_at, updated_at
            "#,
        )
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

    /// Get account by full path (e.g., "Assets:Current Assets:Checking")
    pub async fn get_account_by_path(&self, path: &str) -> Result<Account> {
        let account = sqlx::query_as::<_, Account>(
            r#"
            SELECT 
                id, name, full_path,
                account_type, account_subtype,
                parent_id, symbol, quantity, average_cost, address, 
                purchase_date, purchase_price, currency, is_active, 
                notes, created_at, updated_at
            FROM accounts 
            WHERE full_path = $1 AND is_active = true
            "#,
        )
        .bind(path)
        .fetch_one(&self.pool)
        .await?;

        Ok(account)
    }
}
