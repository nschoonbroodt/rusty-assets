use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Represents a single row in the cash flow statement report.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CashFlowRow {
    pub activity_type: String, // Operating, Investing, Financing
    pub category_name: String, // Parent account name
    pub account_name: String,  // Account name
    pub account_path: String,  // Full account path
    pub cash_flow: Decimal,    // Cash flow amount (positive = inflow, negative = outflow)
}
