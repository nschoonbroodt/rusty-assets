use rust_decimal::Decimal;
use thiserror::Error;
use uuid::Uuid;

use crate::models::account::types::{AccountSubtype, AccountType};

/// Comprehensive validation errors with detailed context
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ValidationError {
    // Account name validation
    #[error("Account name cannot be empty")]
    EmptyName,

    #[error("Account name '{name}' is too long (maximum {max} characters, got {actual})")]
    NameTooLong {
        name: String,
        max: usize,
        actual: usize,
    },

    #[error(
        "Account name '{name}' contains invalid characters (only letters, numbers, spaces, hyphens, and underscores allowed)"
    )]
    InvalidNameCharacters { name: String },

    #[error("Account name '{name}' already exists in this location")]
    DuplicateName { name: String },

    // Currency validation
    #[error("Invalid currency code '{currency}' (must be valid ISO 4217 code)")]
    InvalidCurrency { currency: String },

    // Symbol validation
    #[error("Invalid investment symbol '{symbol}' (must be 1-10 uppercase letters)")]
    InvalidSymbol { symbol: String },

    // Account type/subtype validation
    #[error(
        "Invalid account type and subtype combination: {account_type:?} cannot have subtype {account_subtype:?}"
    )]
    InvalidTypeSubtypeCombination {
        account_type: AccountType,
        account_subtype: AccountSubtype,
    },

    // Investment field validation
    #[error("Investment accounts with symbol must have quantity specified")]
    MissingQuantityForSymbol,

    #[error("Quantity must be positive, got {quantity}")]
    InvalidQuantity { quantity: Decimal },

    #[error("Average cost must be positive, got {cost}")]
    InvalidAverageCost { cost: Decimal },

    // Real estate validation
    #[error("Real estate address cannot be empty")]
    EmptyAddress,

    #[error("Purchase price must be positive, got {price}")]
    InvalidPurchasePrice { price: Decimal },

    // Hierarchy validation
    #[error(
        "Invalid parent-child account type relationship: {parent_type:?} cannot contain {child_type:?}"
    )]
    InvalidHierarchy {
        parent_type: AccountType,
        child_type: AccountType,
    },

    #[error("Account hierarchy too deep (maximum {max} levels, attempting {actual})")]
    HierarchyTooDeep { max: usize, actual: usize },

    #[error("Circular reference detected: account cannot be its own parent")]
    CircularReference,

    // Path validation
    #[error("Account path cannot be empty")]
    EmptyPath,

    #[error("Account path '{path}' contains empty segments")]
    EmptyPathSegments { path: String },

    #[error("Account path '{path}' is too long (maximum {max} characters, got {actual})")]
    PathTooLong {
        path: String,
        max: usize,
        actual: usize,
    },

    // Referential integrity
    #[error("Parent account with ID {parent_id} does not exist")]
    ParentNotFound { parent_id: Uuid },

    #[error("Parent account with ID {parent_id} is inactive")]
    ParentInactive { parent_id: Uuid },

    // Account state validation
    #[error("Cannot deactivate account with active child accounts")]
    HasActiveChildren,

    #[error("Cannot deactivate account with non-zero balance")]
    HasNonZeroBalance,

    #[error("Cannot deactivate account with recent transactions (within {days} days)")]
    HasRecentTransactions { days: u32 },

    // Business rule validation
    #[error(
        "Non-investment accounts cannot have investment fields (symbol, quantity, average_cost)"
    )]
    InvestmentFieldsOnNonInvestment,

    #[error(
        "Non-real-estate accounts cannot have real estate fields (address, purchase_date, purchase_price)"
    )]
    RealEstateFieldsOnNonRealEstate,
}

/// Validation context for collecting multiple errors
#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub errors: Vec<ValidationError>,
}

impl ValidationContext {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn into_result(self) -> Result<(), ValidationError> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            // Return the first error, or could create a compound error
            Err(self.errors.into_iter().next().unwrap())
        }
    }
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self::new()
    }
}
