use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Price history for tracking asset values over time
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PriceHistory {
    pub id: Uuid,
    pub symbol: String,
    pub price: Decimal,
    pub price_date: chrono::NaiveDate,
    pub currency: String,
    pub source: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// New price history entry for creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPriceHistory {
    pub symbol: String,
    pub price: Decimal,
    pub price_date: chrono::NaiveDate,
    pub currency: String,
    pub source: Option<String>,
}
