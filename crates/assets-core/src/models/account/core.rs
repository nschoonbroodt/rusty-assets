use super::types::{AccountSubtype, AccountType};
use bon::Builder;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Chart of Accounts - the foundation of double-entry bookkeeping
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Account {
    pub id: Uuid,
    pub name: String,
    pub account_type: AccountType,
    pub account_subtype: AccountSubtype,
    pub parent_id: Option<Uuid>,
    pub full_path: Option<String>, // Full account path (e.g., "Assets:Current Assets:Checking")

    // Asset-specific fields (null for non-assets)
    pub symbol: Option<String>,        // Stock/ETF symbol
    pub quantity: Option<Decimal>,     // Shares/units owned
    pub average_cost: Option<Decimal>, // Average cost basis

    // Real estate specific
    pub address: Option<String>,
    pub purchase_date: Option<DateTime<Utc>>,
    pub purchase_price: Option<Decimal>,

    // General
    pub currency: String,
    pub is_active: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New account data for creation
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct NewAccount {
    #[builder(into)]
    pub name: String,
    pub account_type: AccountType,
    pub account_subtype: AccountSubtype,
    pub parent_id: Option<Uuid>,

    // Asset-specific fields (optional)
    pub symbol: Option<String>,
    pub quantity: Option<Decimal>,
    pub average_cost: Option<Decimal>,

    // Real estate specific (optional)
    pub address: Option<String>,
    pub purchase_date: Option<DateTime<Utc>>,
    pub purchase_price: Option<Decimal>, // General fields
    #[builder(into, default = "EUR")]
    pub currency: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct NewAccountByPath {
    #[builder(into)]
    pub full_path: String,
    pub account_type: AccountType,
    pub account_subtype: AccountSubtype,

    #[builder(into, default = "EUR")]
    pub currency: String,

    // Asset-specific fields (optional)
    pub symbol: Option<String>,
    pub quantity: Option<Decimal>,
    pub average_cost: Option<Decimal>,

    // Real estate specific (optional)
    pub address: Option<String>,
    pub purchase_date: Option<DateTime<Utc>>,
    pub purchase_price: Option<Decimal>,

    // General fields
    pub notes: Option<String>,
}

impl Account {
    /// Calculate the current balance of this account from journal entries
    pub async fn calculate_balance(&self, pool: &sqlx::PgPool) -> Result<Decimal, sqlx::Error> {
        // This now queries the `account_running_balances` view for the latest balance.
        // It assumes that the view provides the most up-to-date balance.
        // If an account has no transactions, it won't appear in `account_running_balances`,
        // so we COALESCE to 0.00 in that case.
        let result: Option<Decimal> = sqlx::query_scalar(
            r#"
            SELECT running_balance
            FROM account_running_balances
            WHERE account_id = $1
            ORDER BY balance_day DESC
            LIMIT 1
            "#,
        )
        .bind(self.id)
        .fetch_optional(pool) // Use fetch_optional as an account might not have any balance entries yet
        .await?;

        Ok(result.unwrap_or(Decimal::ZERO)) // Return 0 if no balance record is found
    }
    /// Get the account's normal balance sign (positive for debit accounts, negative for credit accounts)
    pub fn normal_balance_sign(&self) -> i32 {
        if self.account_type.increases_with_debit() {
            1
        } else {
            -1
        }
    }
}
