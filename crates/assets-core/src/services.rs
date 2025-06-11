//! Business logic and services for financial operations

use crate::models::{Account, AccountType, Transaction, JournalEntry, NewTransaction, NewJournalEntry, TransactionWithEntries};
use crate::error::Result;
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
    pub async fn create_transaction(&self, new_transaction: NewTransaction) -> Result<TransactionWithEntries> {
        if !new_transaction.is_balanced() {
            return Err(crate::error::CoreError::UnbalancedTransaction {
                expected: Decimal::ZERO,
                actual: new_transaction.entries.iter().map(|e| e.amount).sum(),
            });
        }

        let mut tx = self.pool.begin().await?;        // Insert transaction header
        let transaction_id = Uuid::new_v4();
        let transaction = sqlx::query_as::<_, Transaction>(
            r#"
            INSERT INTO transactions (id, description, reference, transaction_date)
            VALUES ($1, $2, $3, $4)
            RETURNING id, description, reference, transaction_date, created_at
            "#,
        )
        .bind(transaction_id)
        .bind(&new_transaction.description)
        .bind(&new_transaction.reference)
        .bind(new_transaction.transaction_date)
        .fetch_one(&mut *tx)
        .await?;        // Insert journal entries
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
    }    /// Get a transaction with all its journal entries
    pub async fn get_transaction(&self, transaction_id: Uuid) -> Result<Option<TransactionWithEntries>> {
        let transaction = sqlx::query_as::<_, Transaction>(
            "SELECT id, description, reference, transaction_date, created_at FROM transactions WHERE id = $1",
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
    ) -> NewTransaction {
        NewTransaction {
            description,
            reference,
            transaction_date,
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
    }    /// Get all accounts
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
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(accounts)
    }    /// Get accounts by type
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
    }    /// Get account by code
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
}
