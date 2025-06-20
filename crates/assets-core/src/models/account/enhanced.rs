use super::core::Account;
use crate::models::pricing::PriceHistory;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Account with market value calculation (for investments)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountWithMarketValue {
    pub account: Account,
    pub book_value: Decimal,                   // From journal entries
    pub market_value: Option<Decimal>,         // quantity × latest price
    pub unrealized_gain_loss: Option<Decimal>, // market_value - book_value
    pub latest_price: Option<PriceHistory>,
}
