use serde::{Deserialize, Serialize};

/// Account types following double-entry bookkeeping principles
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Copy)]
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
    OwnerEquity,

    // Income subtypes
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
