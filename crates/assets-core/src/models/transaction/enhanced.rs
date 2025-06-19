use super::core::{JournalEntry, Transaction};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A complete transaction with its journal entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionWithEntries {
    pub transaction: Transaction,
    pub entries: Vec<JournalEntry>,
}

/// Journal entry with account information for display purposes
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct JournalEntryWithAccount {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub account_id: Uuid,
    pub account_path: String,
    pub account_name: String,
    pub amount: Decimal,
    pub memo: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Transaction with journal entries including account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionWithEntriesAndAccounts {
    pub transaction: Transaction,
    pub entries: Vec<JournalEntryWithAccount>,
}
