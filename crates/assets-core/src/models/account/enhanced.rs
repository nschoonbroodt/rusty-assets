use super::{
    core::Account,
    ownership::{AccountOwnership, AccountOwnershipWithUser},
};
use crate::models::pricing::PriceHistory;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Enhanced account with ownership information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountWithOwnership {
    pub account: Account,
    pub ownership: Vec<AccountOwnership>,
    pub user_balance: Option<Decimal>, // User's portion of the balance
    pub user_percentage: Option<Decimal>, // User's ownership percentage
}

/// Enhanced account with ownership information including user details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountWithOwnershipAndUsers {
    pub account: Account,
    pub ownership: Vec<AccountOwnershipWithUser>,
    pub user_balance: Option<Decimal>, // User's portion of the balance
    pub user_percentage: Option<Decimal>, // User's ownership percentage
}

/// Account with market value calculation (for investments)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountWithMarketValue {
    pub account: Account,
    pub book_value: Decimal,                   // From journal entries
    pub market_value: Option<Decimal>,         // quantity × latest price
    pub unrealized_gain_loss: Option<Decimal>, // market_value - book_value
    pub latest_price: Option<PriceHistory>,
}
