use super::account_validator::{AccountValidator, ValidationConfig};
use super::errors::ValidationError;
use crate::models::account::{types::*, NewAccount};
use crate::tests::utils::*;
use rust_decimal::Decimal;
use std::str::FromStr;

#[tokio::test]
async fn test_validate_empty_name() {
    let (pool, _container) = setup_test_db().await;
    let validator = AccountValidator::new(pool);

    let account = NewAccount {
        name: "".to_string(), // Empty name should fail
        account_type: AccountType::Asset,
        account_subtype: AccountSubtype::Checking,
        parent_id: None,
        currency: "EUR".to_string(),
        symbol: None,
        quantity: None,
        average_cost: None,
        address: None,
        purchase_date: None,
        purchase_price: None,
        notes: None,
    };

    let result = validator.validate_new_account(&account).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ValidationError::EmptyName));
}

#[tokio::test]
async fn test_validate_invalid_currency() {
    let (pool, _container) = setup_test_db().await;
    let validator = AccountValidator::new(pool);

    let account = NewAccount {
        name: "Test Account".to_string(),
        account_type: AccountType::Asset,
        account_subtype: AccountSubtype::Checking,
        parent_id: None,
        currency: "INVALID".to_string(), // Invalid currency
        symbol: None,
        quantity: None,
        average_cost: None,
        address: None,
        purchase_date: None,
        purchase_price: None,
        notes: None,
    };

    let result = validator.validate_new_account(&account).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ValidationError::InvalidCurrency { .. }
    ));
}

#[tokio::test]
async fn test_validate_invalid_type_subtype_combination() {
    let (pool, _container) = setup_test_db().await;
    let validator = AccountValidator::new(pool);

    let account = NewAccount {
        name: "Test Account".to_string(),
        account_type: AccountType::Asset, // Asset type
        account_subtype: AccountSubtype::Salary, // Income subtype - invalid combination
        parent_id: None,
        currency: "EUR".to_string(),
        symbol: None,
        quantity: None,
        average_cost: None,
        address: None,
        purchase_date: None,
        purchase_price: None,
        notes: None,
    };

    let result = validator.validate_new_account(&account).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ValidationError::InvalidTypeSubtypeCombination { .. }
    ));
}

#[tokio::test]
async fn test_validate_investment_fields_on_non_investment() {
    let (pool, _container) = setup_test_db().await;
    let validator = AccountValidator::new(pool);

    let account = NewAccount {
        name: "Test Account".to_string(),
        account_type: AccountType::Asset,
        account_subtype: AccountSubtype::Checking, // Not an investment account
        parent_id: None,
        currency: "EUR".to_string(),
        symbol: Some("AAPL".to_string()), // Investment field on non-investment account
        quantity: None,
        average_cost: None,
        address: None,
        purchase_date: None,
        purchase_price: None,
        notes: None,
    };

    let result = validator.validate_new_account(&account).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ValidationError::InvestmentFieldsOnNonInvestment
    ));
}

#[tokio::test]
async fn test_validate_real_estate_fields_on_non_real_estate() {
    let (pool, _container) = setup_test_db().await;
    let validator = AccountValidator::new(pool);

    let account = NewAccount {
        name: "Test Account".to_string(),
        account_type: AccountType::Asset,
        account_subtype: AccountSubtype::Checking, // Not real estate
        parent_id: None,
        currency: "EUR".to_string(),
        symbol: None,
        quantity: None,
        average_cost: None,
        address: Some("123 Main St".to_string()), // Real estate field on non-real-estate account
        purchase_date: None,
        purchase_price: None,
        notes: None,
    };

    let result = validator.validate_new_account(&account).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ValidationError::RealEstateFieldsOnNonRealEstate
    ));
}

#[tokio::test]
async fn test_validate_invalid_symbol_format() {
    let (pool, _container) = setup_test_db().await;
    let validator = AccountValidator::new(pool);

    let account = NewAccount {
        name: "Test Investment".to_string(),
        account_type: AccountType::Asset,
        account_subtype: AccountSubtype::Stocks,
        parent_id: None,
        currency: "EUR".to_string(),
        symbol: Some("invalid_symbol_123".to_string()), // Invalid symbol format
        quantity: Some(Decimal::from(10)),
        average_cost: Some(Decimal::from_str("100.00").unwrap()),
        address: None,
        purchase_date: None,
        purchase_price: None,
        notes: None,
    };

    let result = validator.validate_new_account(&account).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ValidationError::InvalidSymbol { .. }
    ));
}

#[tokio::test]
async fn test_validate_missing_quantity_for_symbol() {
    let (pool, _container) = setup_test_db().await;
    let validator = AccountValidator::new(pool);

    let account = NewAccount {
        name: "Test Investment".to_string(),
        account_type: AccountType::Asset,
        account_subtype: AccountSubtype::Stocks,
        parent_id: None,
        currency: "EUR".to_string(),
        symbol: Some("AAPL".to_string()),
        quantity: None, // Missing quantity when symbol is provided
        average_cost: None,
        address: None,
        purchase_date: None,
        purchase_price: None,
        notes: None,
    };

    let result = validator.validate_new_account(&account).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ValidationError::MissingQuantityForSymbol
    ));
}

#[tokio::test]
async fn test_validate_negative_quantity() {
    let (pool, _container) = setup_test_db().await;
    let validator = AccountValidator::new(pool);

    let account = NewAccount {
        name: "Test Investment".to_string(),
        account_type: AccountType::Asset,
        account_subtype: AccountSubtype::Stocks,
        parent_id: None,
        currency: "EUR".to_string(),
        symbol: Some("AAPL".to_string()),
        quantity: Some(Decimal::from(-10)), // Negative quantity
        average_cost: Some(Decimal::from_str("100.00").unwrap()),
        address: None,
        purchase_date: None,
        purchase_price: None,
        notes: None,
    };

    let result = validator.validate_new_account(&account).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ValidationError::InvalidQuantity { .. }
    ));
}

#[tokio::test]
async fn test_validate_valid_account() {
    let (pool, _container) = setup_test_db().await;
    let validator = AccountValidator::new(pool);

    let account = NewAccount {
        name: "Valid Test Account".to_string(),
        account_type: AccountType::Asset,
        account_subtype: AccountSubtype::Checking,
        parent_id: None,
        currency: "EUR".to_string(),
        symbol: None,
        quantity: None,
        average_cost: None,
        address: None,
        purchase_date: None,
        purchase_price: None,
        notes: Some("A valid account".to_string()),
    };

    let result = validator.validate_new_account(&account).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validate_valid_investment_account() {
    let (pool, _container) = setup_test_db().await;
    let validator = AccountValidator::new(pool);

    let account = NewAccount {
        name: "Valid Investment".to_string(),
        account_type: AccountType::Asset,
        account_subtype: AccountSubtype::Stocks,
        parent_id: None,
        currency: "USD".to_string(),
        symbol: Some("AAPL".to_string()),
        quantity: Some(Decimal::from(100)),
        average_cost: Some(Decimal::from_str("150.75").unwrap()),
        address: None,
        purchase_date: None,
        purchase_price: None,
        notes: Some("Apple stock".to_string()),
    };

    let result = validator.validate_new_account(&account).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validation_config_strict_currency_disabled() {
    let (pool, _container) = setup_test_db().await;
    
    let config = ValidationConfig {
        strict_currency_validation: false,
        ..ValidationConfig::default()
    };
    let validator = AccountValidator::with_config(pool, config);

    let account = NewAccount {
        name: "Test Account".to_string(),
        account_type: AccountType::Asset,
        account_subtype: AccountSubtype::Checking,
        parent_id: None,
        currency: "INVALID".to_string(), // Should be allowed with strict_currency_validation = false
        symbol: None,
        quantity: None,
        average_cost: None,
        address: None,
        purchase_date: None,
        purchase_price: None,
        notes: None,
    };

    let result = validator.validate_new_account(&account).await;
    assert!(result.is_ok()); // Should pass with relaxed validation
}