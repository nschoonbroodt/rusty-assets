use super::core::NewJournalEntry;
use bon::Builder;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Helper struct for creating new transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTransaction {
    pub description: String,
    pub reference: Option<String>,
    pub transaction_date: DateTime<Utc>,
    pub entries: Vec<NewJournalEntry>,
    // Import tracking fields
    pub import_source: Option<String>,
    pub import_batch_id: Option<Uuid>,
    pub external_reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct JournalEntryByPath {
    #[builder(into)]
    pub account_path: String,
    pub amount: Decimal,
    pub memo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct NewTransactionByPath {
    #[builder(into)]
    pub description: String,
    pub date: DateTime<Utc>,
    pub entries: Vec<JournalEntryByPath>,
    pub reference: Option<String>,

    pub memo: Option<String>,
}

impl NewTransaction {
    /// Validate that the transaction balances (sum of entries = 0)
    pub fn is_balanced(&self) -> bool {
        self.entries.iter().map(|e| e.amount).sum::<Decimal>() == Decimal::ZERO
    }

    /// Get the total debits (positive amounts)
    pub fn total_debits(&self) -> Decimal {
        self.entries
            .iter()
            .filter(|e| e.amount > Decimal::ZERO)
            .map(|e| e.amount)
            .sum()
    }

    /// Get the total credits (negative amounts, returned as positive)
    pub fn total_credits(&self) -> Decimal {
        self.entries
            .iter()
            .filter(|e| e.amount < Decimal::ZERO)
            .map(|e| -e.amount)
            .sum()
    }
}

impl NewTransactionByPath {
    /// Create a simple two-account transfer
    /// Amount flows FROM from_account TO to_account
    pub fn simple_transfer(
        description: impl Into<String>,
        date: DateTime<Utc>,
        from_account: impl Into<String>,
        to_account: impl Into<String>,
        amount: Decimal, // Always positive
    ) -> Self {
        Self {
            description: description.into(),
            date,
            entries: vec![
                JournalEntryByPath {
                    account_path: to_account.into(),
                    amount, // Positive (receives money)
                    memo: None,
                },
                JournalEntryByPath {
                    account_path: from_account.into(),
                    amount: -amount, // Negative (gives money)
                    memo: None,
                },
            ],
            reference: None,
            memo: None,
        }
    }

    /// Create an income transaction (money flows from income account to asset account)
    pub fn income(
        description: impl Into<String>,
        date: DateTime<Utc>,
        income_account_path: impl Into<String>,
        asset_account_path: impl Into<String>,
        amount: Decimal,
    ) -> Self {
        Self::simple_transfer(
            description,
            date,
            income_account_path, // Income account gives money (credit)
            asset_account_path,  // Asset account receives money (debit)
            amount,
        )
    }

    /// Create an expense transaction (money flows from asset account to expense account)
    pub fn expense(
        description: impl Into<String>,
        date: DateTime<Utc>,
        expense_account_path: impl Into<String>,
        payment_account_path: impl Into<String>, // Could be asset or liability
        amount: Decimal,
    ) -> Self {
        Self::simple_transfer(
            description,
            date,
            payment_account_path, // Payment account gives money
            expense_account_path, // Expense account receives money
            amount,
        )
    }
}
