use super::*;
use crate::tests::utils::*;
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

// Ownership-related tests removed as ownership model has been eliminated

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

// Ownership-related test removed

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

#[tokio::test]
async fn test_update_account_name() {
    let (pool, _container) = setup_test_db().await;
    let service = AccountService::new(pool);

    // Create an account
    let new_account = create_test_new_account();
    let account = service.create_account(new_account).await.unwrap();

    // Update the name
    let updates = AccountUpdates {
        name: Some("Updated Account Name".to_string()),
        ..Default::default()
    };

    let result = service.update_account(account.id, updates).await;
    assert!(result.is_ok());
    let updated_account = result.unwrap();

    assert_eq!(updated_account.name, "Updated Account Name");
    assert_eq!(updated_account.id, account.id);
    assert!(updated_account.updated_at > account.updated_at);
}

#[tokio::test]
async fn test_update_account_investment_fields() {
    let (pool, _container) = setup_test_db().await;
    let service = AccountService::new(pool);

    // Create an investment account
    let investment_account = NewAccount {
        name: "Stock Account".to_string(),
        account_type: AccountType::Asset,
        account_subtype: AccountSubtype::Stocks,
        parent_id: None,
        currency: "EUR".to_string(),
        symbol: Some("AAPL".to_string()),
        quantity: Some(Decimal::from(100)),
        average_cost: Some(Decimal::from_str("150.00").unwrap()),
        address: None,
        purchase_date: None,
        purchase_price: None,
        notes: None,
    };

    let account = service.create_account(investment_account).await.unwrap();

    // Update investment fields
    let updates = AccountUpdates {
        symbol: Some("MSFT".to_string()),
        quantity: Some(Decimal::from(200)),
        average_cost: Some(Decimal::from_str("250.00").unwrap()),
        notes: Some("Updated to Microsoft stock".to_string()),
        ..Default::default()
    };

    let result = service.update_account(account.id, updates).await;
    assert!(result.is_ok());
    let updated_account = result.unwrap();

    assert_eq!(updated_account.symbol, Some("MSFT".to_string()));
    assert_eq!(updated_account.quantity, Some(Decimal::from(200)));
    assert_eq!(
        updated_account.average_cost,
        Some(Decimal::from_str("250.00").unwrap())
    );
    assert_eq!(
        updated_account.notes,
        Some("Updated to Microsoft stock".to_string())
    );
}

#[tokio::test]
async fn test_update_account_no_changes() {
    let (pool, _container) = setup_test_db().await;
    let service = AccountService::new(pool);

    // Create an account
    let new_account = create_test_new_account();
    let account = service.create_account(new_account).await.unwrap();

    // Update with no changes
    let updates = AccountUpdates::default();
    let result = service.update_account(account.id, updates).await;
    assert!(result.is_ok());
    let updated_account = result.unwrap();

    // Should return the same account unchanged
    assert_eq!(updated_account.name, account.name);
    assert_eq!(updated_account.updated_at, account.updated_at);
}

#[tokio::test]
async fn test_update_nonexistent_account() {
    let (pool, _container) = setup_test_db().await;
    let service = AccountService::new(pool);

    let random_id = Uuid::new_v4();
    let updates = AccountUpdates {
        name: Some("New Name".to_string()),
        ..Default::default()
    };

    let result = service.update_account(random_id, updates).await;
    assert!(result.is_err());
    // Should get a database error (no rows found) when trying to fetch_one
}

#[tokio::test]
async fn test_deactivate_account() {
    let (pool, _container) = setup_test_db().await;
    let service = AccountService::new(pool);

    // Create an account
    let new_account = create_test_new_account();
    let account = service.create_account(new_account).await.unwrap();

    // Verify it's active initially
    assert!(account.is_active);

    // Deactivate the account
    let result = service.deactivate_account(account.id).await;
    assert!(result.is_ok());

    // Verify the account is deactivated
    let retrieved = service.get_account(account.id).await.unwrap().unwrap();
    assert!(!retrieved.is_active);

    // Verify it doesn't appear in active account lists
    let all_accounts = service.get_all_accounts().await.unwrap();
    assert!(!all_accounts.iter().any(|a| a.id == account.id));
}

#[tokio::test]
async fn test_deactivate_nonexistent_account() {
    let (pool, _container) = setup_test_db().await;
    let service = AccountService::new(pool);

    let random_id = Uuid::new_v4();
    let result = service.deactivate_account(random_id).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_reactivate_account() {
    let (pool, _container) = setup_test_db().await;
    let service = AccountService::new(pool);

    // Create and deactivate an account
    let new_account = create_test_new_account();
    let account = service.create_account(new_account).await.unwrap();
    service.deactivate_account(account.id).await.unwrap();

    // Reactivate the account
    let result = service.reactivate_account(account.id).await;
    assert!(result.is_ok());
    let reactivated_account = result.unwrap();

    assert!(reactivated_account.is_active);
    assert_eq!(reactivated_account.id, account.id);
    assert_eq!(reactivated_account.name, account.name);

    // Verify it appears in active account lists again
    let all_accounts = service.get_all_accounts().await.unwrap();
    assert!(all_accounts.iter().any(|a| a.id == account.id));
}

#[tokio::test]
async fn test_reactivate_nonexistent_account() {
    let (pool, _container) = setup_test_db().await;
    let service = AccountService::new(pool);

    let random_id = Uuid::new_v4();
    let result = service.reactivate_account(random_id).await;
    assert!(result.is_err());
    // Should fail with database error (account not found)
}

#[tokio::test]
async fn test_get_account_by_path_optional() {
    let (pool, _container) = setup_test_db().await;
    let service = AccountService::new(pool);

    // Test with nonexistent path
    let result = service
        .get_account_by_path_optional("Nonexistent:Path")
        .await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());

    // Create an account and test with existing path
    let account_by_path = NewAccountByPath {
        full_path: "Test:Path".to_string(),
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

    let created_account = service
        .create_account_by_path(account_by_path)
        .await
        .unwrap();

    let result = service.get_account_by_path_optional("Test:Path").await;
    assert!(result.is_ok());
    let found_account = result.unwrap().unwrap();
    assert_eq!(found_account.id, created_account.id);
}

#[tokio::test]
async fn test_account_updates_has_updates() {
    // Test empty updates
    let empty_updates = AccountUpdates::default();
    assert!(!empty_updates.has_updates());

    // Test with name update
    let name_update = AccountUpdates {
        name: Some("New Name".to_string()),
        ..Default::default()
    };
    assert!(name_update.has_updates());

    // Test with multiple updates
    let multiple_updates = AccountUpdates {
        name: Some("New Name".to_string()),
        notes: Some("New notes".to_string()),
        symbol: Some("STOCK".to_string()),
        ..Default::default()
    };
    assert!(multiple_updates.has_updates());
}
