use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Represents a single row in the income statement report.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IncomeStatementRow {
    pub category_name: Option<String>, // Added category_name, optional as some accounts might not have a category
    pub account_name: String,
    pub account_path: String, // Added account_path for full account path display
    pub total_amount: Decimal,
}
