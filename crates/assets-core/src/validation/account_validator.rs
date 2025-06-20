use super::errors::{ValidationContext, ValidationError};
use crate::models::account::{
    Account, NewAccount,
    types::{AccountSubtype, AccountType},
};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::collections::HashSet;
use uuid::Uuid;

/// Configuration for validation behavior
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub max_name_length: usize,
    pub max_path_length: usize,
    pub max_hierarchy_depth: usize,
    pub strict_currency_validation: bool,
    pub require_investment_fields: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_name_length: 100,
            max_path_length: 500,
            max_hierarchy_depth: 10,
            strict_currency_validation: true,
            require_investment_fields: true,
        }
    }
}

/// Main validator for account operations
pub struct AccountValidator {
    pool: PgPool,
    config: ValidationConfig,
}

impl AccountValidator {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            config: ValidationConfig::default(),
        }
    }

    pub fn with_config(pool: PgPool, config: ValidationConfig) -> Self {
        Self { pool, config }
    }

    /// Validate a new account for creation
    pub async fn validate_new_account(&self, account: &NewAccount) -> Result<(), ValidationError> {
        let mut context = ValidationContext::new();

        // Basic field validation
        self.validate_name(&account.name, &mut context);
        self.validate_currency(&account.currency, &mut context);
        self.validate_type_subtype_combination(
            account.account_type,
            account.account_subtype,
            &mut context,
        );

        // Investment-specific validation
        if self.is_investment_account(account.account_subtype) {
            self.validate_investment_fields(
                &account.symbol,
                account.quantity,
                account.average_cost,
                &mut context,
            );
        } else {
            self.validate_no_investment_fields(
                &account.symbol,
                account.quantity,
                account.average_cost,
                &mut context,
            );
        }

        // Real estate-specific validation
        if account.account_subtype == AccountSubtype::RealEstate {
            self.validate_real_estate_fields(
                &account.address,
                account.purchase_price,
                &mut context,
            );
        } else {
            self.validate_no_real_estate_fields(
                &account.address,
                account.purchase_date,
                account.purchase_price,
                &mut context,
            );
        }

        // Hierarchy validation (if parent specified)
        if let Some(parent_id) = account.parent_id {
            self.validate_hierarchy(parent_id, account.account_type, &mut context)
                .await;
            self.validate_name_uniqueness_in_parent(&account.name, Some(parent_id), &mut context)
                .await;
        } else {
            self.validate_name_uniqueness_in_parent(&account.name, None, &mut context)
                .await;
        }

        context.into_result()
    }

    /// Validate account updates
    pub async fn validate_account_updates(
        &self,
        account_id: Uuid,
        updates: &crate::services::AccountUpdates,
    ) -> Result<(), ValidationError> {
        let mut context = ValidationContext::new();

        // Get existing account to validate updates against
        let existing_account_result = sqlx::query_as::<_, Account>(
            "SELECT id, name, full_path, account_type, account_subtype, parent_id, symbol, quantity, average_cost, address, purchase_date, purchase_price, currency, is_active, notes, created_at, updated_at FROM accounts WHERE id = $1"
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await;

        let existing_account = match existing_account_result {
            Ok(Some(account)) => account,
            Ok(None) => {
                context.add_error(ValidationError::ParentNotFound {
                    parent_id: account_id,
                });
                return context.into_result();
            }
            Err(_) => {
                context.add_error(ValidationError::ParentNotFound {
                    parent_id: account_id,
                });
                return context.into_result();
            }
        };

        // Validate name if being updated
        if let Some(ref name) = updates.name {
            self.validate_name(name, &mut context);

            // Check uniqueness within parent (only if name is actually changing)
            if name != &existing_account.name {
                self.validate_name_uniqueness_in_parent(
                    name,
                    existing_account.parent_id,
                    &mut context,
                )
                .await;
            }
        }

        // Validate currency if being updated
        if let Some(ref currency) = updates.currency {
            self.validate_currency(currency, &mut context);
        }

        // Validate investment fields updates
        if updates.symbol.is_some() || updates.quantity.is_some() || updates.average_cost.is_some()
        {
            let is_investment = self.is_investment_account(existing_account.account_subtype);

            if is_investment {
                // Merge current values with updates for validation
                let symbol = updates.symbol.as_ref().or(existing_account.symbol.as_ref());
                let quantity = updates.quantity.or(existing_account.quantity);
                let average_cost = updates.average_cost.or(existing_account.average_cost);

                self.validate_investment_fields(
                    &symbol.cloned(),
                    quantity,
                    average_cost,
                    &mut context,
                );
            } else {
                // Non-investment account shouldn't have investment fields
                self.validate_no_investment_fields(
                    &updates.symbol,
                    updates.quantity,
                    updates.average_cost,
                    &mut context,
                );
            }
        }

        // Validate real estate fields updates
        if updates.address.is_some()
            || updates.purchase_date.is_some()
            || updates.purchase_price.is_some()
        {
            if existing_account.account_subtype == AccountSubtype::RealEstate {
                // Merge current values with updates for validation
                let address = updates
                    .address
                    .as_ref()
                    .or(existing_account.address.as_ref());
                let purchase_price = updates.purchase_price.or(existing_account.purchase_price);

                self.validate_real_estate_fields(&address.cloned(), purchase_price, &mut context);
            } else {
                // Non-real-estate account shouldn't have real estate fields
                self.validate_no_real_estate_fields(
                    &updates.address,
                    updates.purchase_date,
                    updates.purchase_price,
                    &mut context,
                );
            }
        }

        context.into_result()
    }

    /// Validate account name
    fn validate_name(&self, name: &str, context: &mut ValidationContext) {
        if name.is_empty() {
            context.add_error(ValidationError::EmptyName);
            return;
        }

        if name.len() > self.config.max_name_length {
            context.add_error(ValidationError::NameTooLong {
                name: name.to_string(),
                max: self.config.max_name_length,
                actual: name.len(),
            });
        }

        // Allow letters, numbers, spaces, hyphens, underscores, and common punctuation
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || " -_.,()".contains(c))
        {
            context.add_error(ValidationError::InvalidNameCharacters {
                name: name.to_string(),
            });
        }
    }

    /// Validate currency code
    fn validate_currency(&self, currency: &str, context: &mut ValidationContext) {
        if !self.config.strict_currency_validation {
            return;
        }

        // Basic ISO 4217 validation - 3 uppercase letters
        if currency.len() != 3 || !currency.chars().all(|c| c.is_ascii_uppercase()) {
            context.add_error(ValidationError::InvalidCurrency {
                currency: currency.to_string(),
            });
            return;
        }

        // Common currency codes validation
        let valid_currencies: HashSet<&str> = [
            "USD", "EUR", "GBP", "JPY", "CHF", "CAD", "AUD", "NZD", "SEK", "NOK", "DKK", "PLN",
            "CZK", "HUF", "BGN", "RON", "HRK", "RUB", "CNY", "INR", "KRW", "SGD", "HKD", "THB",
            "MXN", "BRL", "ZAR", "TRY", "ILS", "AED", "SAR", "QAR",
        ]
        .into_iter()
        .collect();

        if !valid_currencies.contains(currency) {
            context.add_error(ValidationError::InvalidCurrency {
                currency: currency.to_string(),
            });
        }
    }

    /// Validate account type and subtype combination
    fn validate_type_subtype_combination(
        &self,
        account_type: AccountType,
        account_subtype: AccountSubtype,
        context: &mut ValidationContext,
    ) {
        let valid = match account_type {
            AccountType::Asset => matches!(
                account_subtype,
                AccountSubtype::Cash
                    | AccountSubtype::Checking
                    | AccountSubtype::Savings
                    | AccountSubtype::InvestmentAccount
                    | AccountSubtype::Stocks
                    | AccountSubtype::Etf
                    | AccountSubtype::Bonds
                    | AccountSubtype::MutualFund
                    | AccountSubtype::Crypto
                    | AccountSubtype::RealEstate
                    | AccountSubtype::Equipment
                    | AccountSubtype::OtherAsset
                    | AccountSubtype::Category
            ),
            AccountType::Liability => matches!(
                account_subtype,
                AccountSubtype::CreditCard
                    | AccountSubtype::Loan
                    | AccountSubtype::Mortgage
                    | AccountSubtype::OtherLiability
                    | AccountSubtype::Category
            ),
            AccountType::Equity => matches!(
                account_subtype,
                AccountSubtype::OpeningBalance
                    | AccountSubtype::RetainedEarnings
                    | AccountSubtype::OwnerEquity
                    | AccountSubtype::Category
            ),
            AccountType::Income => matches!(
                account_subtype,
                AccountSubtype::Salary
                    | AccountSubtype::Bonus
                    | AccountSubtype::Dividend
                    | AccountSubtype::Interest
                    | AccountSubtype::Investment
                    | AccountSubtype::Rental
                    | AccountSubtype::CapitalGains
                    | AccountSubtype::OtherIncome
                    | AccountSubtype::Category
            ),
            AccountType::Expense => matches!(
                account_subtype,
                AccountSubtype::Food
                    | AccountSubtype::Housing
                    | AccountSubtype::Transportation
                    | AccountSubtype::Communication
                    | AccountSubtype::Entertainment
                    | AccountSubtype::Personal
                    | AccountSubtype::Utilities
                    | AccountSubtype::Healthcare
                    | AccountSubtype::Taxes
                    | AccountSubtype::Fees
                    | AccountSubtype::OtherExpense
                    | AccountSubtype::Category
            ),
        };

        if !valid {
            context.add_error(ValidationError::InvalidTypeSubtypeCombination {
                account_type,
                account_subtype,
            });
        }
    }

    /// Check if this is an investment account subtype
    fn is_investment_account(&self, subtype: AccountSubtype) -> bool {
        matches!(
            subtype,
            AccountSubtype::InvestmentAccount
                | AccountSubtype::Stocks
                | AccountSubtype::Etf
                | AccountSubtype::Bonds
                | AccountSubtype::MutualFund
                | AccountSubtype::Crypto
        )
    }

    /// Validate investment fields for investment accounts
    fn validate_investment_fields(
        &self,
        symbol: &Option<String>,
        quantity: Option<Decimal>,
        average_cost: Option<Decimal>,
        context: &mut ValidationContext,
    ) {
        if let Some(sym) = symbol {
            self.validate_symbol(sym, context);

            // If symbol is provided, quantity should be provided too
            if self.config.require_investment_fields && quantity.is_none() {
                context.add_error(ValidationError::MissingQuantityForSymbol);
            }
        }

        if let Some(qty) = quantity {
            if qty <= Decimal::ZERO {
                context.add_error(ValidationError::InvalidQuantity { quantity: qty });
            }
        }

        if let Some(cost) = average_cost {
            if cost <= Decimal::ZERO {
                context.add_error(ValidationError::InvalidAverageCost { cost });
            }
        }
    }

    /// Validate that non-investment accounts don't have investment fields
    fn validate_no_investment_fields(
        &self,
        symbol: &Option<String>,
        quantity: Option<Decimal>,
        average_cost: Option<Decimal>,
        context: &mut ValidationContext,
    ) {
        if symbol.is_some() || quantity.is_some() || average_cost.is_some() {
            context.add_error(ValidationError::InvestmentFieldsOnNonInvestment);
        }
    }

    /// Validate investment symbol format
    fn validate_symbol(&self, symbol: &str, context: &mut ValidationContext) {
        if symbol.is_empty() || symbol.len() > 10 {
            context.add_error(ValidationError::InvalidSymbol {
                symbol: symbol.to_string(),
            });
            return;
        }

        // Symbols should be uppercase letters and possibly numbers/dots
        if !symbol
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '.')
        {
            context.add_error(ValidationError::InvalidSymbol {
                symbol: symbol.to_string(),
            });
        }
    }

    /// Validate real estate fields
    fn validate_real_estate_fields(
        &self,
        address: &Option<String>,
        purchase_price: Option<Decimal>,
        context: &mut ValidationContext,
    ) {
        if let Some(addr) = address {
            if addr.trim().is_empty() {
                context.add_error(ValidationError::EmptyAddress);
            }
        }

        if let Some(price) = purchase_price {
            if price <= Decimal::ZERO {
                context.add_error(ValidationError::InvalidPurchasePrice { price });
            }
        }
    }

    /// Validate that non-real-estate accounts don't have real estate fields
    fn validate_no_real_estate_fields(
        &self,
        address: &Option<String>,
        purchase_date: Option<chrono::DateTime<chrono::Utc>>,
        purchase_price: Option<Decimal>,
        context: &mut ValidationContext,
    ) {
        if address.is_some() || purchase_date.is_some() || purchase_price.is_some() {
            context.add_error(ValidationError::RealEstateFieldsOnNonRealEstate);
        }
    }

    /// Validate account hierarchy
    async fn validate_hierarchy(
        &self,
        parent_id: Uuid,
        child_type: AccountType,
        context: &mut ValidationContext,
    ) {
        // Check if parent exists and is active
        let parent_result = sqlx::query_as::<_, Account>(
            "SELECT id, name, full_path, account_type, account_subtype, parent_id, symbol, quantity, average_cost, address, purchase_date, purchase_price, currency, is_active, notes, created_at, updated_at FROM accounts WHERE id = $1"
        )
        .bind(parent_id)
        .fetch_optional(&self.pool)
        .await;

        match parent_result {
            Ok(Some(parent)) => {
                if !parent.is_active {
                    context.add_error(ValidationError::ParentInactive { parent_id });
                }

                // Validate parent-child type relationship
                self.validate_parent_child_types(parent.account_type, child_type, context);

                // Check hierarchy depth
                // TODO: Implement depth checking
            }
            Ok(None) => {
                context.add_error(ValidationError::ParentNotFound { parent_id });
            }
            Err(_) => {
                // Database error - let it bubble up later
                context.add_error(ValidationError::ParentNotFound { parent_id });
            }
        }
    }

    /// Validate parent-child account type relationships
    fn validate_parent_child_types(
        &self,
        parent_type: AccountType,
        child_type: AccountType,
        context: &mut ValidationContext,
    ) {
        // In double-entry bookkeeping, accounts of the same type can generally contain each other
        // Different types should not be mixed in hierarchy
        if parent_type != child_type {
            context.add_error(ValidationError::InvalidHierarchy {
                parent_type,
                child_type,
            });
        }
    }

    /// Validate name uniqueness within parent
    async fn validate_name_uniqueness_in_parent(
        &self,
        name: &str,
        parent_id: Option<Uuid>,
        context: &mut ValidationContext,
    ) {
        let query = if parent_id.is_some() {
            "SELECT COUNT(*) FROM accounts WHERE name = $1 AND parent_id = $2 AND is_active = true"
        } else {
            "SELECT COUNT(*) FROM accounts WHERE name = $1 AND parent_id IS NULL AND is_active = true"
        };

        let count_result: Result<i64, sqlx::Error> = if let Some(pid) = parent_id {
            sqlx::query_scalar(query)
                .bind(name)
                .bind(pid)
                .fetch_one(&self.pool)
                .await
        } else {
            sqlx::query_scalar(query)
                .bind(name)
                .fetch_one(&self.pool)
                .await
        };

        match count_result {
            Ok(count) if count > 0 => {
                context.add_error(ValidationError::DuplicateName {
                    name: name.to_string(),
                });
            }
            Err(_) => {
                // Database error - will be handled elsewhere
            }
            _ => {} // No duplicates found
        }
    }
}
