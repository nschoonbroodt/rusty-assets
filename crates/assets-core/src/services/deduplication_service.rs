use crate::error::Result;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TransactionMatch {
    pub id: Uuid,
    pub primary_transaction_id: Uuid,
    pub duplicate_transaction_id: Uuid,
    pub match_confidence: Decimal,
    pub match_criteria: serde_json::Value,
    pub match_type: MatchType,
    pub status: MatchStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionComparisonDetails {
    pub id: Uuid,
    pub description: String,
    pub transaction_date: DateTime<Utc>,
    pub import_source: Option<String>,
    pub entries_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "UPPERCASE")]
pub enum MatchType {
    Exact,
    Probable,
    Possible,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "UPPERCASE")]
pub enum MatchStatus {
    Pending,
    Confirmed,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PotentialDuplicate {
    pub potential_duplicate_id: Uuid,
    pub match_confidence: Decimal,
    pub match_criteria: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TransactionWithDuplicateInfo {
    pub id: Uuid,
    pub description: String,
    pub transaction_date: DateTime<Utc>,
    pub import_source: Option<String>,
    pub import_batch_id: Option<Uuid>,
    pub amount: Decimal,
    pub duplicate_count: i64,
    pub has_duplicates: bool,
}

pub struct DeduplicationService {
    pool: PgPool,
}

impl DeduplicationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find a transaction by partial UUID (useful for CLI commands)
    pub async fn find_transaction_by_partial_uuid(
        &self,
        partial_uuid: &str,
    ) -> Result<Option<Uuid>> {
        let query = format!(
            "SELECT id FROM transactions WHERE id::text LIKE '{}%' LIMIT 1",
            partial_uuid
        );
        let result = sqlx::query_scalar::<_, Uuid>(&query)
            .fetch_optional(&self.pool)
            .await?;

        Ok(result)
    }

    /// Find a transaction match by partial UUID (useful for CLI commands)
    pub async fn find_match_by_partial_uuid(&self, partial_uuid: &str) -> Result<Option<Uuid>> {
        let query = format!(
            "SELECT id FROM transaction_matches WHERE id::text LIKE '{}%' LIMIT 1",
            partial_uuid
        );
        let result = sqlx::query_scalar::<_, Uuid>(&query)
            .fetch_optional(&self.pool)
            .await?;

        Ok(result)
    }

    /// Find potential duplicates for a given transaction
    pub async fn find_potential_duplicates(
        &self,
        transaction_id: Uuid,
        amount_tolerance: Option<Decimal>,
        date_tolerance_days: Option<i32>,
    ) -> Result<Vec<PotentialDuplicate>> {
        let amount_tolerance = amount_tolerance.unwrap_or(Decimal::from_str("0.01").unwrap());
        let date_tolerance_days = date_tolerance_days.unwrap_or(3);

        let duplicates = sqlx::query_as::<_, PotentialDuplicate>(
            r#"
            SELECT 
                potential_duplicate_id,
                match_confidence,
                match_criteria
            FROM fn_find_potential_duplicates($1, $2, $3)
            "#,
        )
        .bind(transaction_id)
        .bind(amount_tolerance)
        .bind(date_tolerance_days)
        .fetch_all(&self.pool)
        .await?;

        Ok(duplicates)
    }
    /// Create a transaction match record
    pub async fn create_transaction_match(
        &self,
        primary_transaction_id: Uuid,
        duplicate_transaction_id: Uuid,
        match_confidence: Decimal,
        match_criteria: serde_json::Value,
        match_type: MatchType,
    ) -> Result<TransactionMatch> {
        let transaction_match = sqlx::query_as::<_, TransactionMatch>(
            r#"
            INSERT INTO transaction_matches 
            (primary_transaction_id, duplicate_transaction_id, match_confidence, match_criteria, match_type, status)
            VALUES ($1, $2, $3, $4, $5, 'PENDING')
            RETURNING id, primary_transaction_id, duplicate_transaction_id, 
                      match_confidence, match_criteria, match_type, 
                      status, created_at, updated_at
            "#
        )
        .bind(primary_transaction_id)
        .bind(duplicate_transaction_id)
        .bind(match_confidence)
        .bind(match_criteria)
        .bind(match_type)
        .fetch_one(&self.pool)
        .await?;

        Ok(transaction_match)
    }
    /// Update match status (confirm or reject)
    pub async fn update_match_status(
        &self,
        match_id: Uuid,
        status: MatchStatus,
    ) -> Result<TransactionMatch> {
        let updated_match = sqlx::query_as::<_, TransactionMatch>(
            r#"
            UPDATE transaction_matches 
            SET status = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING id, primary_transaction_id, duplicate_transaction_id, 
                      match_confidence, match_criteria, match_type, 
                      status, created_at, updated_at
            "#,
        )
        .bind(match_id)
        .bind(status)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_match)
    }
    /// Get all transactions with their duplicate information
    pub async fn get_transactions_with_duplicates(
        &self,
        limit: Option<i32>,
        only_with_duplicates: bool,
    ) -> Result<Vec<TransactionWithDuplicateInfo>> {
        let transactions = if only_with_duplicates {
            sqlx::query_as::<_, TransactionWithDuplicateInfo>(
                r#"
                SELECT id, description, transaction_date, import_source, import_batch_id, 
                       amount, duplicate_count, has_duplicates
                FROM v_transactions_with_duplicates
                WHERE has_duplicates = true
                ORDER BY transaction_date DESC
                LIMIT $1
                "#,
            )
            .bind(limit.unwrap_or(100))
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, TransactionWithDuplicateInfo>(
                r#"
                SELECT id, description, transaction_date, import_source, import_batch_id, 
                       amount, duplicate_count, has_duplicates
                FROM v_transactions_with_duplicates
                ORDER BY transaction_date DESC
                LIMIT $1
                "#,
            )
            .bind(limit.unwrap_or(100))
            .fetch_all(&self.pool)
            .await?
        };

        Ok(transactions)
    }
    /// Get all matches for a transaction
    pub async fn get_matches_for_transaction(
        &self,
        transaction_id: Uuid,
    ) -> Result<Vec<TransactionMatch>> {
        let matches = sqlx::query_as::<_, TransactionMatch>(
            r#"
            SELECT id, primary_transaction_id, duplicate_transaction_id, 
                   match_confidence, match_criteria, match_type, 
                   status, created_at, updated_at
            FROM transaction_matches
            WHERE primary_transaction_id = $1 OR duplicate_transaction_id = $1
            ORDER BY match_confidence DESC
            "#,
        )
        .bind(transaction_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(matches)
    }
    /// Run automatic duplicate detection on all transactions from a specific import batch
    pub async fn detect_duplicates_for_batch(
        &self,
        import_batch_id: Uuid,
        auto_confirm_exact_matches: bool,
    ) -> Result<Vec<TransactionMatch>> {
        // Get all transactions from this batch
        let batch_transactions =
            sqlx::query("SELECT id FROM transactions WHERE import_batch_id = $1")
                .bind(import_batch_id)
                .fetch_all(&self.pool)
                .await?;
        let mut created_matches = Vec::new();

        for tx_row in batch_transactions {
            let tx_id: Uuid = tx_row.get("id");
            let potential_duplicates = self.find_potential_duplicates(tx_id, None, None).await?;

            for duplicate in potential_duplicates {
                // Only create matches above a minimum confidence threshold
                if duplicate.match_confidence >= Decimal::from_str("0.6").unwrap() {
                    let match_type =
                        if duplicate.match_confidence >= Decimal::from_str("0.95").unwrap() {
                            MatchType::Exact
                        } else if duplicate.match_confidence >= Decimal::from_str("0.8").unwrap() {
                            MatchType::Probable
                        } else {
                            MatchType::Possible
                        };

                    let mut transaction_match = self
                        .create_transaction_match(
                            tx_id,
                            duplicate.potential_duplicate_id,
                            duplicate.match_confidence,
                            duplicate.match_criteria,
                            match_type.clone(),
                        )
                        .await?;

                    // Auto-confirm exact matches if requested
                    if auto_confirm_exact_matches && matches!(match_type, MatchType::Exact) {
                        transaction_match = self
                            .update_match_status(transaction_match.id, MatchStatus::Confirmed)
                            .await?;
                    }

                    created_matches.push(transaction_match);
                }
            }
        }

        Ok(created_matches)
    }
    /// Merge two transactions by marking one as duplicate and hiding it from reports
    pub async fn merge_duplicate_transactions(
        &self,
        primary_transaction_id: Uuid,
        duplicate_transaction_id: Uuid,
    ) -> Result<()> {
        // Use the database function to hide the duplicate transaction
        sqlx::query("SELECT fn_hide_duplicate_transaction($1, $2)")
            .bind(duplicate_transaction_id)
            .bind(primary_transaction_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Unhide a previously merged transaction (undo the merge)
    pub async fn unhide_duplicate_transaction(&self, transaction_id: Uuid) -> Result<()> {
        // Use the database function to unhide the transaction
        sqlx::query("SELECT fn_unhide_duplicate_transaction($1)")
            .bind(transaction_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Convenience method for CLI - merge transactions by IDs
    pub async fn merge_transaction(&self, primary_id: Uuid, duplicate_id: Uuid) -> Result<()> {
        self.merge_duplicate_transactions(primary_id, duplicate_id)
            .await
    }

    /// Convenience method for CLI - unmerge transaction by ID
    pub async fn unmerge_transaction(&self, transaction_id: Uuid) -> Result<()> {
        self.unhide_duplicate_transaction(transaction_id).await
    }

    /// Get detailed transaction information for duplicate comparison
    pub async fn get_transaction_details_for_comparison(
        &self,
        transaction_id: Uuid,
    ) -> Result<Option<TransactionComparisonDetails>> {
        let query = r#"
            SELECT 
                t.id,
                t.description,
                t.transaction_date,
                t.import_source,
                COALESCE(
                    (SELECT string_agg(a.full_path || ': ' || je.amount, ' | ' ORDER BY je.amount DESC)
                     FROM journal_entries je 
                     JOIN accounts a ON je.account_id = a.id 
                     WHERE je.transaction_id = t.id),
                    'No entries'
                ) as entries_summary
            FROM transactions t
            WHERE t.id = $1
        "#;

        let result = sqlx::query(query)
            .bind(transaction_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = result {
            Ok(Some(TransactionComparisonDetails {
                id: row.get("id"),
                description: row.get("description"),
                transaction_date: row.get("transaction_date"),
                import_source: row.get("import_source"),
                entries_summary: row.get("entries_summary"),
            }))
        } else {
            Ok(None)
        }
    }
}

// Helper for string conversions
use std::str::FromStr;
