use crate::error::Result;
use crate::models::{
    JournalEntry, JournalEntryWithAccount, NewJournalEntry, NewTransaction, Transaction,
    TransactionWithEntries, TransactionWithEntriesAndAccounts,
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

    /// Get a transaction with all its journal entries including account information
    pub async fn get_transaction_with_accounts(
        &self,
        transaction_id: Uuid,
    ) -> Result<Option<TransactionWithEntriesAndAccounts>> {
        let transaction = sqlx::query_as::<_, Transaction>(
            "SELECT id, description, reference, transaction_date, created_by, created_at FROM transactions WHERE id = $1",
        )
        .bind(transaction_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(transaction) = transaction else {
            return Ok(None);
        };

        let entries = sqlx::query_as::<_, JournalEntryWithAccount>(
            r#"
            SELECT 
                je.id, 
                je.transaction_id, 
                je.account_id, 
                a.full_path as account_path,
                a.name as account_name,
                je.amount, 
                je.memo, 
                je.created_at 
            FROM journal_entries je
            INNER JOIN accounts a ON je.account_id = a.id
            WHERE je.transaction_id = $1 
            ORDER BY je.created_at
            "#,
        )
        .bind(transaction_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(Some(TransactionWithEntriesAndAccounts {
            transaction,
            entries,
        }))
    }

    /// Get transactions with optional filtering
    pub async fn get_transactions_with_filters(
        &self,
        from_date: Option<DateTime<Utc>>,
        to_date: Option<DateTime<Utc>>,
        account_path: Option<&str>,
        user_id: Option<Uuid>,
        limit: u32,
    ) -> Result<Vec<TransactionWithEntries>> {
        // Build the base query
        let mut query = String::from(
            r#"
            SELECT DISTINCT t.id, t.description, t.reference, t.transaction_date, t.created_by, t.created_at
            FROM transactions t
            "#,
        );

        let mut conditions = Vec::new();
        let mut bind_index = 1;

        // Join with journal entries and accounts if needed
        if account_path.is_some() || user_id.is_some() {
            query.push_str(" INNER JOIN journal_entries je ON t.id = je.transaction_id");
            query.push_str(" INNER JOIN accounts a ON je.account_id = a.id");

            if user_id.is_some() {
                query.push_str(" INNER JOIN account_ownership ao ON a.id = ao.account_id");
            }
        }

        query.push_str(" WHERE 1=1");

        // Add date filters
        if from_date.is_some() {
            conditions.push(format!(" AND t.transaction_date >= ${}", bind_index));
            bind_index += 1;
        }

        if to_date.is_some() {
            conditions.push(format!(" AND t.transaction_date <= ${}", bind_index));
            bind_index += 1;
        }

        // Add account path filter
        if account_path.is_some() {
            conditions.push(format!(" AND a.full_path LIKE ${}", bind_index));
            bind_index += 1;
        }

        // Add user filter
        if user_id.is_some() {
            conditions.push(format!(" AND ao.user_id = ${}", bind_index));
            bind_index += 1;
        }

        // Add all conditions to query
        for condition in conditions {
            query.push_str(&condition);
        }

        query.push_str(" ORDER BY t.transaction_date DESC, t.created_at DESC");
        query.push_str(&format!(" LIMIT ${}", bind_index));

        // Execute the query with proper parameter binding
        let mut query_builder = sqlx::query_as::<_, Transaction>(&query);

        if let Some(from) = from_date {
            query_builder = query_builder.bind(from);
        }
        if let Some(to) = to_date {
            query_builder = query_builder.bind(to);
        }
        if let Some(path) = account_path {
            query_builder = query_builder.bind(format!("{}%", path));
        }
        if let Some(uid) = user_id {
            query_builder = query_builder.bind(uid);
        }
        query_builder = query_builder.bind(limit as i64);

        let transactions = query_builder.fetch_all(&self.pool).await?;

        // Fetch journal entries for each transaction
        let mut result = Vec::new();
        for transaction in transactions {
            let entries = sqlx::query_as::<_, JournalEntry>(
                "SELECT id, transaction_id, account_id, amount, memo, created_at FROM journal_entries WHERE transaction_id = $1 ORDER BY created_at",
            )
            .bind(transaction.id)
            .fetch_all(&self.pool)
            .await?;

            result.push(TransactionWithEntries {
                transaction,
                entries,
            });
        }

        Ok(result)
    }

    /// Get transactions with optional filtering, including account information
    pub async fn get_transactions_with_filters_and_accounts(
        &self,
        from_date: Option<DateTime<Utc>>,
        to_date: Option<DateTime<Utc>>,
        account_path: Option<&str>,
        user_id: Option<Uuid>,
        limit: u32,
    ) -> Result<Vec<TransactionWithEntriesAndAccounts>> {
        // Build the base query
        let mut query = String::from(
            r#"
            SELECT DISTINCT t.id, t.description, t.reference, t.transaction_date, t.created_by, t.created_at
            FROM transactions t
            "#,
        );

        let mut conditions = Vec::new();
        let mut bind_index = 1;

        // Join with journal entries and accounts if needed
        if account_path.is_some() || user_id.is_some() {
            query.push_str(" INNER JOIN journal_entries je ON t.id = je.transaction_id");
            query.push_str(" INNER JOIN accounts a ON je.account_id = a.id");

            if user_id.is_some() {
                query.push_str(" INNER JOIN account_ownership ao ON a.id = ao.account_id");
            }
        }

        query.push_str(" WHERE 1=1");

        // Add date filters
        if from_date.is_some() {
            conditions.push(format!(" AND t.transaction_date >= ${}", bind_index));
            bind_index += 1;
        }

        if to_date.is_some() {
            conditions.push(format!(" AND t.transaction_date <= ${}", bind_index));
            bind_index += 1;
        }

        // Add account path filter
        if account_path.is_some() {
            conditions.push(format!(" AND a.full_path LIKE ${}", bind_index));
            bind_index += 1;
        }

        // Add user filter
        if user_id.is_some() {
            conditions.push(format!(" AND ao.user_id = ${}", bind_index));
            bind_index += 1;
        }

        // Add all conditions to query
        for condition in conditions {
            query.push_str(&condition);
        }

        query.push_str(" ORDER BY t.transaction_date DESC, t.created_at DESC");
        query.push_str(&format!(" LIMIT ${}", bind_index));

        // Execute the query with proper parameter binding
        let mut query_builder = sqlx::query_as::<_, Transaction>(&query);

        if let Some(from) = from_date {
            query_builder = query_builder.bind(from);
        }
        if let Some(to) = to_date {
            query_builder = query_builder.bind(to);
        }
        if let Some(path) = account_path {
            query_builder = query_builder.bind(format!("{}%", path));
        }
        if let Some(uid) = user_id {
            query_builder = query_builder.bind(uid);
        }
        query_builder = query_builder.bind(limit as i64);

        let transactions = query_builder.fetch_all(&self.pool).await?;

        // Fetch journal entries with account information for each transaction
        let mut result = Vec::new();
        for transaction in transactions {
            let entries = sqlx::query_as::<_, JournalEntryWithAccount>(
                r#"
                SELECT 
                    je.id, 
                    je.transaction_id, 
                    je.account_id, 
                    a.full_path as account_path,
                    a.name as account_name,
                    je.amount, 
                    je.memo, 
                    je.created_at 
                FROM journal_entries je
                INNER JOIN accounts a ON je.account_id = a.id
                WHERE je.transaction_id = $1 
                ORDER BY je.created_at
                "#,
            )
            .bind(transaction.id)
            .fetch_all(&self.pool)
            .await?;

            result.push(TransactionWithEntriesAndAccounts {
                transaction,
                entries,
            });
        }

        Ok(result)
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
