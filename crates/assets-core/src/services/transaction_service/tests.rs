use super::*;
use crate::CoreError;
use crate::models::{AccountSubtype, AccountType, NewJournalEntry, NewTransaction};
use crate::services::AccountService;
use crate::tests::utils::*;
use chrono::Utc;
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

#[tokio::test]
async fn test_create_balanced_transaction() {
    let (pool, _container) = setup_test_db().await;
    let tx_service = TransactionService::new(pool.clone());
    let account_service = AccountService::new(pool);

    // Create test accounts
    let checking_account = account_service
        .create_account(create_test_new_account_with_type(
            "Checking",
            AccountType::Asset,
            AccountSubtype::Checking,
        ))
        .await
        .unwrap();

    let groceries_account = account_service
        .create_account(create_test_new_account_with_type(
            "Groceries",
            AccountType::Expense,
            AccountSubtype::Food,
        ))
        .await
        .unwrap();

    // Create balanced transaction: Groceries expense paid from checking
    let new_transaction = NewTransaction {
        description: "Grocery shopping".to_string(),
        transaction_date: Utc::now(),
        reference: None,
        created_by: None,
        entries: vec![
            NewJournalEntry {
                account_id: groceries_account.id,
                amount: Decimal::from_str("50.00").unwrap(), // Debit expense
                memo: None,
            },
            NewJournalEntry {
                account_id: checking_account.id,
                amount: Decimal::from_str("-50.00").unwrap(), // Credit asset (money leaves)
                memo: None,
            },
        ],
        import_source: None,
        import_batch_id: None,
        external_reference: None,
    };

    // Should succeed because transaction is balanced
    let result = tx_service.create_transaction(new_transaction).await;
    assert!(result.is_ok());

    let transaction = result.unwrap();
    assert_eq!(transaction.entries.len(), 2);
    assert_eq!(transaction.transaction.description, "Grocery shopping");

    // Verify the transaction is actually balanced
    let total: Decimal = transaction.entries.iter().map(|e| e.amount).sum();
    assert_eq!(total, Decimal::ZERO);
}

#[tokio::test]
async fn test_reject_unbalanced_transaction() {
    let (pool, _container) = setup_test_db().await;
    let tx_service = TransactionService::new(pool.clone());
    let account_service = AccountService::new(pool);

    // Create test accounts
    let checking_account = account_service
        .create_account(create_test_new_account_with_type(
            "Checking",
            AccountType::Asset,
            AccountSubtype::Checking,
        ))
        .await
        .unwrap();

    let groceries_account = account_service
        .create_account(create_test_new_account_with_type(
            "Groceries",
            AccountType::Expense,
            AccountSubtype::Food,
        ))
        .await
        .unwrap();

    // Create UNBALANCED transaction
    let unbalanced_transaction = NewTransaction {
        description: "Unbalanced transaction".to_string(),
        transaction_date: Utc::now(),
        reference: None,
        created_by: None,
        entries: vec![
            NewJournalEntry {
                account_id: groceries_account.id,
                amount: Decimal::from_str("50.00").unwrap(), // Debit expense
                memo: None,
            },
            NewJournalEntry {
                account_id: checking_account.id,
                amount: Decimal::from_str("-25.00").unwrap(), // Credit only half the amount - UNBALANCED!
                memo: None,
            },
        ],
        import_source: None,
        import_batch_id: None,
        external_reference: None,
    };

    // Should fail with UnbalancedTransaction error
    let result = tx_service.create_transaction(unbalanced_transaction).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        CoreError::UnbalancedTransaction { expected, actual } => {
            assert_eq!(expected, Decimal::ZERO);
            assert_eq!(actual, Decimal::from_str("25.00").unwrap()); // 50.00 + (-25.00) = 25.00
        }
        _ => panic!("Expected UnbalancedTransaction error"),
    }
}

#[tokio::test]
async fn test_new_transaction_is_balanced_method() {
    // Test the is_balanced() method on NewTransaction

    // Balanced transaction
    let balanced = NewTransaction {
        description: "Test".to_string(),
        transaction_date: Utc::now(),
        reference: None,
        created_by: None,
        entries: vec![
            NewJournalEntry {
                account_id: Uuid::new_v4(),
                amount: Decimal::from_str("100.00").unwrap(),
                memo: None,
            },
            NewJournalEntry {
                account_id: Uuid::new_v4(),
                amount: Decimal::from_str("-100.00").unwrap(),
                memo: None,
            },
        ],
        import_source: None,
        import_batch_id: None,
        external_reference: None,
    };
    assert!(balanced.is_balanced());

    // Unbalanced transaction
    let unbalanced = NewTransaction {
        description: "Test".to_string(),
        transaction_date: Utc::now(),
        reference: None,
        created_by: None,
        entries: vec![
            NewJournalEntry {
                account_id: Uuid::new_v4(),
                amount: Decimal::from_str("100.00").unwrap(),
                memo: None,
            },
            NewJournalEntry {
                account_id: Uuid::new_v4(),
                amount: Decimal::from_str("-50.00").unwrap(), // Unbalanced
                memo: None,
            },
        ],
        import_source: None,
        import_batch_id: None,
        external_reference: None,
    };
    assert!(!unbalanced.is_balanced());
}

#[tokio::test]
async fn test_transaction_total_debits_credits() {
    let transaction = NewTransaction {
        description: "Test".to_string(),
        transaction_date: Utc::now(),
        reference: None,
        created_by: None,
        entries: vec![
            NewJournalEntry {
                account_id: Uuid::new_v4(),
                amount: Decimal::from_str("100.00").unwrap(), // Debit
                memo: None,
            },
            NewJournalEntry {
                account_id: Uuid::new_v4(),
                amount: Decimal::from_str("50.00").unwrap(), // Debit
                memo: None,
            },
            NewJournalEntry {
                account_id: Uuid::new_v4(),
                amount: Decimal::from_str("-75.00").unwrap(), // Credit
                memo: None,
            },
            NewJournalEntry {
                account_id: Uuid::new_v4(),
                amount: Decimal::from_str("-75.00").unwrap(), // Credit
                memo: None,
            },
        ],
        import_source: None,
        import_batch_id: None,
        external_reference: None,
    };

    assert_eq!(
        transaction.total_debits(),
        Decimal::from_str("150.00").unwrap()
    ); // 100 + 50
    assert_eq!(
        transaction.total_credits(),
        Decimal::from_str("150.00").unwrap()
    ); // 75 + 75 (returned as positive)
    assert!(transaction.is_balanced());
}

#[tokio::test]
async fn test_simple_transaction_helpers() {
    // Test the static helper methods
    let debit_account = Uuid::new_v4();
    let credit_account = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let amount = Decimal::from_str("250.00").unwrap();
    let date = Utc::now();

    let simple = TransactionService::create_simple_transaction(
        "Simple transfer".to_string(),
        debit_account,
        credit_account,
        amount,
        date,
        Some("REF-123".to_string()),
        Some(user_id),
    );

    assert_eq!(simple.description, "Simple transfer");
    assert_eq!(simple.reference, Some("REF-123".to_string()));
    assert_eq!(simple.created_by, Some(user_id));
    assert_eq!(simple.transaction_date, date);
    assert_eq!(simple.entries.len(), 2);

    // Verify debit entry
    let debit_entry = simple
        .entries
        .iter()
        .find(|e| e.account_id == debit_account)
        .unwrap();
    assert_eq!(debit_entry.amount, amount);

    // Verify credit entry
    let credit_entry = simple
        .entries
        .iter()
        .find(|e| e.account_id == credit_account)
        .unwrap();
    assert_eq!(credit_entry.amount, -amount);

    // Verify transaction is balanced
    assert!(simple.is_balanced());
}

#[tokio::test]
async fn test_double_entry_bookkeeping_principles() {
    let (pool, _container) = setup_test_db().await;
    let tx_service = TransactionService::new(pool.clone());
    let account_service = AccountService::new(pool);

    // Create accounts for testing double-entry principles
    let checking = account_service
        .create_account(create_test_new_account_with_type(
            "Checking",
            AccountType::Asset,
            AccountSubtype::Checking,
        ))
        .await
        .unwrap();

    let salary = account_service
        .create_account(create_test_new_account_with_type(
            "Salary",
            AccountType::Income,
            AccountSubtype::Salary,
        ))
        .await
        .unwrap();

    let rent = account_service
        .create_account(create_test_new_account_with_type(
            "Rent",
            AccountType::Expense,
            AccountSubtype::Housing,
        ))
        .await
        .unwrap();

    // Test 1: Income transaction (Salary -> Checking)
    // In double-entry: Debit Asset (increases), Credit Income (increases income)
    let salary_transaction = NewTransaction {
        description: "Monthly salary".to_string(),
        transaction_date: Utc::now(),
        reference: None,
        created_by: None,
        entries: vec![
            NewJournalEntry {
                account_id: checking.id,
                amount: Decimal::from_str("5000.00").unwrap(), // Debit asset (money comes in)
                memo: None,
            },
            NewJournalEntry {
                account_id: salary.id,
                amount: Decimal::from_str("-5000.00").unwrap(), // Credit income (income increases)
                memo: None,
            },
        ],
        import_source: None,
        import_batch_id: None,
        external_reference: None,
    };

    assert!(salary_transaction.is_balanced());
    let result = tx_service.create_transaction(salary_transaction).await;
    assert!(result.is_ok(), "Salary transaction should succeed");

    // Test 2: Expense transaction (Checking -> Rent)
    // In double-entry: Credit Asset (decreases), Debit Expense (increases expense)
    let rent_transaction = NewTransaction {
        description: "Monthly rent".to_string(),
        transaction_date: Utc::now(),
        reference: None,
        created_by: None,
        entries: vec![
            NewJournalEntry {
                account_id: rent.id,
                amount: Decimal::from_str("1500.00").unwrap(), // Debit expense (expense increases)
                memo: None,
            },
            NewJournalEntry {
                account_id: checking.id,
                amount: Decimal::from_str("-1500.00").unwrap(), // Credit asset (money goes out)
                memo: None,
            },
        ],
        import_source: None,
        import_batch_id: None,
        external_reference: None,
    };

    assert!(rent_transaction.is_balanced());
    let result = tx_service.create_transaction(rent_transaction).await;
    assert!(result.is_ok(), "Rent transaction should succeed");
}
