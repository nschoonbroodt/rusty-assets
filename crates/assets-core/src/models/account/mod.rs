//! Account-related models and types
//!
//! This module contains all types related to account management:
//! - Core account structures (Account, NewAccount)
//! - Account type definitions (AccountType, AccountSubtype)
//! - Ownership models (AccountOwnership, fractional ownership)
//! - Enhanced account types with ownership and market value information
//! - User context for filtering operations

pub mod context;
pub mod core;
pub mod enhanced;
pub mod ownership;
pub mod types;

// Re-export all public types for easier importing
pub use context::*;
pub use core::*;
pub use enhanced::*;
pub use ownership::*;
pub use types::*;
