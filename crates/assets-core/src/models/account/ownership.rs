use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Account ownership for fractional ownership support
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AccountOwnership {
    pub id: Uuid,
    pub user_id: Uuid,
    pub account_id: Uuid,
    pub ownership_percentage: Decimal,
    pub created_at: DateTime<Utc>,
}

/// Account ownership with user information - avoids multiple database round trips
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AccountOwnershipWithUser {
    pub id: Uuid,
    pub user_id: Uuid,
    pub account_id: Uuid,
    pub ownership_percentage: Decimal,
    pub created_at: DateTime<Utc>,

    // User information from JOIN
    pub user_name: String,
    pub user_display_name: String,
    pub user_is_active: bool,
}
