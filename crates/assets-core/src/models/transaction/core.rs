use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Transaction header - groups related journal entries
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub description: String,
    pub reference: Option<String>, // Check number, transfer ID, etc.
    pub transaction_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    // Import tracking fields
    pub import_source: Option<String>,
    pub import_batch_id: Option<Uuid>,
    pub external_reference: Option<String>,
    // Duplicate tracking fields
    pub is_duplicate: Option<bool>,
    pub merged_into_transaction_id: Option<Uuid>,
}

/// Journal entries - the actual debits and credits that make up a transaction
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct JournalEntry {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub account_id: Uuid,
    pub amount: Decimal, // Positive for debits, negative for credits
    pub memo: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Helper struct for creating new journal entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewJournalEntry {
    pub account_id: Uuid,
    pub amount: Decimal,
    pub memo: Option<String>,
}
