use crate::error::Result;
use crate::models::{
    Account, AccountOwnership, AccountOwnershipWithUser, AccountType, AccountWithOwnership,
    AccountWithOwnershipAndUsers, NewAccount,
};
use crate::{AccountSubtype, NewAccountByPath};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

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
    }
    /// Get accounts by type
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
    }
    /// Get account by ID
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
                COALESCE(u.display_name, u.name) as user_display_name,
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
    /// Create a new account with default ownership (100% to first user)
    pub async fn create_account(&self, new_account: NewAccount) -> Result<Account> {
        // Get the first user as default owner
        let user_service = crate::services::UserService::new(self.pool.clone());
        let default_user = user_service.get_first_user().await?;

        match default_user {
            Some(user) => {
                // Create account with default 100% ownership to first user
                let ownership = vec![(user.id, Decimal::from(1))]; // 100%
                self.create_account_with_ownership(new_account, ownership)
                    .await
            }
            None => {
                // No users exist - create account without ownership for now
                // This should be rare and might indicate a setup issue
                self.create_account_without_ownership(new_account).await
            }
        }
    }

    /// Create an account by path, auto-creating missing parent accounts
    pub async fn create_account_by_path(&self, account: NewAccountByPath) -> Result<Account> {
        // Parse the path into components
        let path_parts: Vec<&str> = account.full_path.split(':').collect();

        if path_parts.is_empty() || path_parts.iter().all(|s| s.trim().is_empty()) {
            return Err(crate::CoreError::EmptyAccountName);
        }

        let mut current_parent_id: Option<Uuid> = None;
        let mut current_path = String::new();

        // Create or find each level of the hierarchy
        for (i, part) in path_parts.iter().enumerate() {
            if i > 0 {
                current_path.push(':');
            }
            current_path.push_str(part);

            // Check if this level already exists
            if let Ok(existing_account) = self.get_account_by_path(&current_path).await {
                current_parent_id = Some(existing_account.id);
                continue;
            }

            // If this is the final part, create with specified type/subtype
            if i == path_parts.len() - 1 {
                let new_account = NewAccount {
                    name: part.to_string(),
                    account_type: account.account_type,
                    account_subtype: account.account_subtype,
                    parent_id: current_parent_id,
                    currency: account.currency,
                    symbol: account.symbol,
                    quantity: account.quantity,
                    average_cost: account.average_cost,
                    address: account.address,
                    purchase_date: account.purchase_date,
                    purchase_price: account.purchase_price,
                    notes: account.notes,
                };

                return self.create_account(new_account).await;
            } else {
                // Create intermediate account as Category
                let intermediate_account = NewAccount {
                    name: part.to_string(),
                    account_type: account.account_type, // Same type as final account
                    account_subtype: AccountSubtype::Category,
                    parent_id: current_parent_id,
                    currency: account.currency.clone(),
                    symbol: None,
                    quantity: None,
                    average_cost: None,
                    address: None,
                    purchase_date: None,
                    purchase_price: None,
                    notes: None,
                };

                let created_account = self.create_account(intermediate_account).await?;
                current_parent_id = Some(created_account.id);
            }
        }

        unreachable!("Should have returned in the loop")
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
        .bind(new_account.account_type)
        .bind(new_account.account_subtype)
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

        let mut tx = self.pool.begin().await?; // Create the account
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
        .bind(new_account.account_type)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::helpers::*;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use std::str::FromStr;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_account_without_users() {
        let (pool, _container) = setup_test_db().await;
        let service = AccountService::new(pool);

        let new_account = create_test_new_account();
        let result = service.create_account(new_account).await;

        assert!(result.is_ok());
        let account = result.unwrap();
        assert_eq!(account.name, "Test Account");
        assert_eq!(account.account_type, AccountType::Asset);
        assert_eq!(account.account_subtype, AccountSubtype::Checking);
        assert_eq!(account.currency, "EUR");
        assert!(account.is_active);
    }

    #[tokio::test]
    async fn test_create_account_with_user() {
        let (pool, _container) = setup_test_db().await;
        let user = create_test_user(&pool).await;
        let service = AccountService::new(pool);

        let new_account = create_test_new_account();
        let result = service.create_account(new_account).await;

        assert!(result.is_ok());
        let account = result.unwrap();

        // Verify account was created
        assert_eq!(account.name, "Test Account");

        // Verify ownership was created
        let ownership_result = service.get_account_with_ownership(account.id).await;
        assert!(ownership_result.is_ok());
        let account_with_ownership = ownership_result.unwrap().unwrap();
        assert_eq!(account_with_ownership.ownership.len(), 1);
        assert_eq!(account_with_ownership.ownership[0].user_id, user.id);
        assert_eq!(
            account_with_ownership.ownership[0].ownership_percentage,
            Decimal::from(1)
        );
    }

    #[tokio::test]
    async fn test_create_account_with_custom_ownership() {
        let (pool, _container) = setup_test_db().await;
        let user1 = create_test_user(&pool).await;

        // Create second user
        let user2 = create_test_user_with_names(&pool, "user2", "User 2").await;

        let service = AccountService::new(pool);

        let new_account = create_test_new_account();
        let ownership = vec![
            (user1.id, Decimal::from_str("0.6").unwrap()), // 60%
            (user2.id, Decimal::from_str("0.4").unwrap()), // 40%
        ];

        let result = service
            .create_account_with_ownership(new_account, ownership)
            .await;

        assert!(result.is_ok());
        let account = result.unwrap();

        // Verify ownership
        let ownership_result = service.get_account_with_ownership(account.id).await;
        assert!(ownership_result.is_ok());
        let account_with_ownership = ownership_result.unwrap().unwrap();
        assert_eq!(account_with_ownership.ownership.len(), 2);

        // Should be ordered by percentage DESC
        assert_eq!(account_with_ownership.ownership[0].user_id, user1.id);
        assert_eq!(
            account_with_ownership.ownership[0].ownership_percentage,
            Decimal::from_str("0.6").unwrap()
        );
        assert_eq!(account_with_ownership.ownership[1].user_id, user2.id);
        assert_eq!(
            account_with_ownership.ownership[1].ownership_percentage,
            Decimal::from_str("0.4").unwrap()
        );
    }

    #[tokio::test]
    async fn test_create_account_with_invalid_ownership() {
        let (pool, _container) = setup_test_db().await;
        let user = create_test_user(&pool).await;
        let service = AccountService::new(pool);

        let new_account = create_test_new_account();
        let invalid_ownership = vec![(user.id, Decimal::from_str("1.5").unwrap())]; // 150%

        let result = service
            .create_account_with_ownership(new_account, invalid_ownership)
            .await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Total ownership percentage cannot exceed 100%")
        );
    }

    #[tokio::test]
    async fn test_get_account_by_id() {
        let (pool, _container) = setup_test_db().await;
        let service = AccountService::new(pool);

        // Create an account
        let new_account = create_test_new_account();
        let created_account = service.create_account(new_account).await.unwrap();

        // Retrieve by ID
        let result = service.get_account(created_account.id).await;
        assert!(result.is_ok());
        let retrieved_account = result.unwrap().unwrap();
        assert_eq!(retrieved_account.id, created_account.id);
        assert_eq!(retrieved_account.name, "Test Account");
    }

    #[tokio::test]
    async fn test_get_nonexistent_account() {
        let (pool, _container) = setup_test_db().await;
        let service = AccountService::new(pool);

        let random_id = Uuid::new_v4();
        let result = service.get_account(random_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_all_accounts() {
        let (pool, _container) = setup_test_db().await;
        let service = AccountService::new(pool);

        // Create multiple accounts
        let account1 = create_test_new_account();
        let mut account2 = create_test_new_account();
        account2.name = "Second Account".to_string();

        service.create_account(account1).await.unwrap();
        service.create_account(account2).await.unwrap();

        // Get all accounts
        let result = service.get_all_accounts().await;
        assert!(result.is_ok());
        let accounts = result.unwrap();
        assert_eq!(accounts.len(), 2);

        // Should be ordered by name
        assert_eq!(accounts[0].name, "Second Account");
        assert_eq!(accounts[1].name, "Test Account");
    }

    #[tokio::test]
    async fn test_get_accounts_by_type() {
        let (pool, _container) = setup_test_db().await;
        let service = AccountService::new(pool);

        // Create accounts of different types
        let asset_account = create_test_new_account(); // Default is Asset
        let mut liability_account = create_test_new_account();
        liability_account.name = "Credit Card".to_string();
        liability_account.account_type = AccountType::Liability;
        liability_account.account_subtype = AccountSubtype::CreditCard;

        service.create_account(asset_account).await.unwrap();
        service.create_account(liability_account).await.unwrap();

        // Get only Asset accounts
        let result = service.get_accounts_by_type(AccountType::Asset).await;
        assert!(result.is_ok());
        let assets = result.unwrap();
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].account_type, AccountType::Asset);

        // Get only Liability accounts
        let result = service.get_accounts_by_type(AccountType::Liability).await;
        assert!(result.is_ok());
        let liabilities = result.unwrap();
        assert_eq!(liabilities.len(), 1);
        assert_eq!(liabilities[0].account_type, AccountType::Liability);
    }

    #[tokio::test]
    async fn test_create_investment_account() {
        let (pool, _container) = setup_test_db().await;
        let service = AccountService::new(pool);

        let investment_account = NewAccount {
            name: "Apple Stock".to_string(),
            account_type: AccountType::Asset,
            account_subtype: AccountSubtype::Stocks,
            parent_id: None,
            currency: "EUR".to_string(),
            symbol: Some("AAPL".to_string()),
            quantity: Some(Decimal::from(100)),
            average_cost: Some(Decimal::from_str("150.50").unwrap()),
            address: None,
            purchase_date: Some(Utc::now()),
            purchase_price: Some(Decimal::from_str("15050.00").unwrap()),
            notes: Some("Technology stock".to_string()),
        };

        let result = service.create_account(investment_account).await;
        assert!(result.is_ok());
        let account = result.unwrap();

        assert_eq!(account.symbol, Some("AAPL".to_string()));
        assert_eq!(account.quantity, Some(Decimal::from(100)));
        assert_eq!(
            account.average_cost,
            Some(Decimal::from_str("150.50").unwrap())
        );
        assert_eq!(account.notes, Some("Technology stock".to_string()));
    }

    #[tokio::test]
    async fn test_create_account_by_path_simple() {
        let (pool, _container) = setup_test_db().await;
        let service = AccountService::new(pool);

        let account_by_path = NewAccountByPath {
            full_path: "Assets:Checking".to_string(),
            account_type: AccountType::Asset,
            account_subtype: AccountSubtype::Checking,
            currency: "EUR".to_string(),
            symbol: None,
            quantity: None,
            average_cost: None,
            address: None,
            purchase_date: None,
            purchase_price: None,
            notes: None,
        };

        let result = service.create_account_by_path(account_by_path).await;
        assert!(result.is_ok());
        let account = result.unwrap();

        assert_eq!(account.name, "Checking");
        assert_eq!(account.account_type, AccountType::Asset);
        assert_eq!(account.account_subtype, AccountSubtype::Checking);

        // Verify we can retrieve it by path
        let path_result = service.get_account_by_path("Assets:Checking").await;
        assert!(path_result.is_ok());
        let retrieved = path_result.unwrap();
        assert_eq!(retrieved.id, account.id);
    }

    #[tokio::test]
    async fn test_create_account_by_path_with_hierarchy() {
        let (pool, _container) = setup_test_db().await;
        let service = AccountService::new(pool);

        let account_by_path = NewAccountByPath {
            full_path: "Assets:Current Assets:Bank:Checking Account".to_string(),
            account_type: AccountType::Asset,
            account_subtype: AccountSubtype::Checking,
            currency: "EUR".to_string(),
            symbol: None,
            quantity: None,
            average_cost: None,
            address: None,
            purchase_date: None,
            purchase_price: None,
            notes: None,
        };

        let result = service.create_account_by_path(account_by_path).await;
        assert!(result.is_ok());
        let final_account = result.unwrap();

        assert_eq!(final_account.name, "Checking Account");
        assert_eq!(final_account.account_subtype, AccountSubtype::Checking);

        // Verify intermediate accounts were created as categories
        let assets_result = service.get_account_by_path("Assets").await;
        assert!(assets_result.is_ok());
        let assets_account = assets_result.unwrap();
        assert_eq!(assets_account.account_subtype, AccountSubtype::Category);

        let current_assets_result = service.get_account_by_path("Assets:Current Assets").await;
        assert!(current_assets_result.is_ok());
        let current_assets = current_assets_result.unwrap();
        assert_eq!(current_assets.account_subtype, AccountSubtype::Category);
        assert_eq!(current_assets.parent_id, Some(assets_account.id));

        // Final account should have correct parent
        assert!(final_account.parent_id.is_some());
    }

    #[tokio::test]
    async fn test_get_account_with_ownership_and_users() {
        let (pool, _container) = setup_test_db().await;
        let user = create_test_user(&pool).await;
        let service = AccountService::new(pool);

        let new_account = create_test_new_account();
        let account = service.create_account(new_account).await.unwrap();

        let result = service
            .get_account_with_ownership_and_users(account.id)
            .await;
        assert!(result.is_ok());
        let account_with_users = result.unwrap().unwrap();

        assert_eq!(account_with_users.ownership.len(), 1);
        assert_eq!(account_with_users.ownership[0].user_id, user.id);
        assert_eq!(account_with_users.ownership[0].user_name, "test_user");
        assert_eq!(
            account_with_users.ownership[0].user_display_name,
            "Test User".to_string()
        );
    }

    #[tokio::test]
    async fn test_empty_account_path() {
        let (pool, _container) = setup_test_db().await;
        let service = AccountService::new(pool);

        let account_by_path = NewAccountByPath {
            full_path: "".to_string(),
            account_type: AccountType::Asset,
            account_subtype: AccountSubtype::Checking,
            currency: "EUR".to_string(),
            symbol: None,
            quantity: None,
            average_cost: None,
            address: None,
            purchase_date: None,
            purchase_price: None,
            notes: None,
        };

        let result = service.create_account_by_path(account_by_path).await;
        assert!(result.is_err());
        // Should return EmptyAccountName error
    }
}
