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
    MutualFund,
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
    Category, // Added Category subtype
}

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

/// User context for filtering operations
#[derive(Debug, Clone, PartialEq)]
pub enum UserContext {
    User(Uuid), // Specific user view
    Family,     // Combined family view
}

/// New account data for creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAccount {
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
    pub purchase_price: Option<Decimal>,

    // General fields
    pub currency: String,
    pub notes: Option<String>,
}

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

/// Account with market value calculation (for investments)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountWithMarketValue {
    pub account: Account,
    pub book_value: Decimal,                   // From journal entries
    pub market_value: Option<Decimal>,         // quantity × latest price
    pub unrealized_gain_loss: Option<Decimal>, // market_value - book_value
    pub latest_price: Option<PriceHistory>,
}

/// Represents a single row in the income statement report.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IncomeStatementRow {
    pub category_name: Option<String>, // Added category_name, optional as some accounts might not have a category
    pub account_name: String,
    pub account_path: String, // Added account_path for full account path display
    pub total_amount: Decimal,
}

/// Journal entry with account information for display purposes
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct JournalEntryWithAccount {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub account_id: Uuid,
    pub account_path: String,
    pub account_name: String,
    pub amount: Decimal,
    pub memo: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Transaction with journal entries including account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionWithEntriesAndAccounts {
    pub transaction: Transaction,
    pub entries: Vec<JournalEntryWithAccount>,
}

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
