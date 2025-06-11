use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Account types following double-entry bookkeeping principles
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "account_type", rename_all = "lowercase")]
pub enum AccountType {
    Asset,     // Cash, investments, real estate, equipment
    Liability, // Credit cards, loans, mortgages
    Equity,    // Owner's equity, retained earnings
    Income,    // Salary, dividends, capital gains
    Expense,   // Food, utilities, taxes, fees
}

/// Account subtypes for more specific categorization
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "account_subtype", rename_all = "snake_case")]
pub enum AccountSubtype {
    // Asset subtypes
    Cash,
    Checking,
    Savings,
    InvestmentAccount,
    Stocks,
    Etf,
    Bonds,
    Crypto,
    RealEstate,
    Equipment,
    OtherAsset,

    // Liability subtypes
    CreditCard,
    Loan,
    Mortgage,
    OtherLiability,

    // Equity subtypes
    OpeningBalance,
    RetainedEarnings,
    OwnerEquity, // Income subtypes
    Salary,
    Bonus,
    Dividend,
    Interest,
    Investment,
    Rental,
    CapitalGains,
    OtherIncome,

    // Expense subtypes
    Food,
    Housing,
    Transportation,
    Communication,
    Entertainment,
    Personal,
    Utilities,
    Healthcare,
    Taxes,
    Fees,
    OtherExpense,
}

/// Chart of Accounts - the foundation of double-entry bookkeeping
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Account {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub account_type: AccountType,
    pub account_subtype: AccountSubtype,
    pub parent_id: Option<Uuid>,

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

/// Transaction header - groups related journal entries
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub description: String,
    pub reference: Option<String>, // Check number, transfer ID, etc.
    pub transaction_date: DateTime<Utc>,
    pub created_by: Option<Uuid>, // User who created the transaction
    pub created_at: DateTime<Utc>,
}

/// Journal entries - the actual debits and credits that make up a transaction
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct JournalEntry {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub account_id: Uuid,
    pub amount: Decimal, // Positive for debits, negative for credits
    pub memo: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// A complete transaction with its journal entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionWithEntries {
    pub transaction: Transaction,
    pub entries: Vec<JournalEntry>,
}

/// Helper struct for creating new transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTransaction {
    pub description: String,
    pub reference: Option<String>,
    pub transaction_date: DateTime<Utc>,
    pub created_by: Option<Uuid>, // User creating the transaction
    pub entries: Vec<NewJournalEntry>,
}

/// Helper struct for creating new journal entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewJournalEntry {
    pub account_id: Uuid,
    pub amount: Decimal,
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

impl AccountType {
    /// Returns true if this account type increases with debits
    pub fn increases_with_debit(&self) -> bool {
        matches!(self, AccountType::Asset | AccountType::Expense)
    }

    /// Returns true if this account type increases with credits
    pub fn increases_with_credit(&self) -> bool {
        matches!(
            self,
            AccountType::Liability | AccountType::Equity | AccountType::Income
        )
    }
}

impl Account {
    /// Calculate the current balance of this account from journal entries
    pub async fn calculate_balance(&self, pool: &sqlx::PgPool) -> Result<Decimal, sqlx::Error> {
        let result: Option<Decimal> = sqlx::query_scalar(
            "SELECT COALESCE(SUM(amount), 0) FROM journal_entries WHERE account_id = $1",
        )
        .bind(self.id)
        .fetch_one(pool)
        .await?;

        Ok(result.unwrap_or(Decimal::ZERO))
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

/// User entity for multi-user support
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Account ownership for fractional ownership support
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AccountOwnership {
    pub id: Uuid,
    pub user_id: Uuid,
    pub account_id: Uuid,
    pub ownership_percentage: Decimal,
    pub created_at: DateTime<Utc>,
}

/// Enhanced account with ownership information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountWithOwnership {
    pub account: Account,
    pub ownership: Vec<AccountOwnership>,
    pub user_balance: Option<Decimal>, // User's portion of the balance
    pub user_percentage: Option<Decimal>, // User's ownership percentage
}

/// User context for filtering operations
#[derive(Debug, Clone, PartialEq)]
pub enum UserContext {
    User(Uuid), // Specific user view
    Family,     // Combined family view
}

impl UserContext {
    pub fn is_family(&self) -> bool {
        matches!(self, UserContext::Family)
    }

    pub fn user_id(&self) -> Option<Uuid> {
        match self {
            UserContext::User(id) => Some(*id),
            UserContext::Family => None,
        }
    }
}
