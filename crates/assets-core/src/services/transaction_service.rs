use crate::error::Result;
use crate::models::{
    JournalEntry, NewJournalEntry, NewTransaction, Transaction, TransactionWithEntries,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

/// View model for transaction displays with account information
#[derive(Debug, Clone)]
pub struct TransactionWithAccountView {
    pub id: Uuid,
    pub description: String,
    pub transaction_date: DateTime<Utc>,
    pub amount: Decimal,
    pub account_name: String,
}

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

    /// Get recent transactions with account information for display purposes
    pub async fn get_recent_transactions_with_accounts(&self, limit: i64) -> Result<Vec<TransactionWithAccountView>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                t.id, 
                t.description, 
                t.transaction_date, 
                je.amount,
                a.name as account_name
            FROM 
                transactions t
            INNER JOIN 
                journal_entries je ON t.id = je.transaction_id
            INNER JOIN
                accounts a ON je.account_id = a.id
            ORDER BY 
                t.transaction_date DESC, t.created_at DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut transaction_views = Vec::with_capacity(rows.len());
        
        for row in rows {
            transaction_views.push(TransactionWithAccountView {
                id: row.id,
                description: row.description,
                transaction_date: row.transaction_date,
                amount: row.amount,
                account_name: row.account_name,
            });
        }
        
        Ok(transaction_views)
    }
}
