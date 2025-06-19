use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a single row in the account ledger report.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AccountLedgerRow {
    pub transaction_date: chrono::NaiveDate,
    pub transaction_id: Uuid,
    pub description: String,
    pub reference: String,
    pub memo: String,
    pub debit_amount: Decimal,
    pub credit_amount: Decimal,
    pub running_balance: Decimal,
}
