//! RustyAssets Data Models
//!
//! This module contains all data models organized by domain:
//! - `account`: Account management, types, and enhanced views
//! - `transaction`: Transaction processing, journal entries, and builders
//! - `pricing`: Asset pricing and market data
//! - `reports`: Financial reporting structures
//! - `import`: Data import tracking and management
//!
//! ## Design Principles
//!
//! - **Domain separation**: Each module focuses on a specific business domain
//! - **Clear dependencies**: Modules avoid circular dependencies
//! - **Re-exports**: All public types are re-exported for backward compatibility
//! - **Documentation**: Each module is well-documented with examples

pub mod account;
pub mod import;
pub mod pricing;
pub mod reports;
pub mod transaction;
pub mod user;

// Re-export all public types to maintain API compatibility
// This ensures existing code continues to work: `use crate::models::Account;`

// Account types
pub use account::{
    Account, AccountSubtype, AccountType, AccountWithMarketValue, NewAccount, NewAccountByPath,
};

// Transaction types
pub use transaction::{
    JournalEntry, JournalEntryByPath, JournalEntryWithAccount, NewJournalEntry, NewTransaction,
    NewTransactionByPath, Transaction, TransactionWithEntries, TransactionWithEntriesAndAccounts,
};

// Pricing types
pub use pricing::{NewPriceHistory, PriceHistory};

// Report types
pub use reports::{AccountLedgerRow, CashFlowRow, IncomeStatementRow};

// Import types
pub use import::{ImportedFile, NewImportedFile};
