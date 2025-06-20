# AccountService Validation Implementation Plan

## Motivation

RustyAssets currently has a critical gap in data validation. The AccountService accepts and processes any input without validation, which poses significant risks for a financial tracking application:

- **Data integrity risks**: Invalid accounts can break double-entry bookkeeping principles
- **User experience issues**: Unclear error messages when operations fail
- **System reliability**: No protection against malformed or malicious input
- **Financial accuracy**: No enforcement of business rules for account hierarchies

This is particularly dangerous for financial software where data accuracy is paramount.

## Current State Analysis

**Validation Gaps Found:**
- **Zero input validation** in AccountService methods
- **No business rule enforcement** for account hierarchies  
- **No currency/symbol validation**
- **No account name constraints** (length, uniqueness, format)
- **Only basic error handling** (CoreError::EmptyAccountName for paths)
- **Transaction validation exists** but not integrated with accounts

**Existing Validation Patterns:**
- Transaction balancing validation in `NewTransaction::is_balanced()`
- Some error types in CoreError (ValidationError, InvalidInput)
- Basic path parsing in `create_account_by_path`

## Implementation Plan

### Phase 1: Validation Framework (High Priority)
1. **Create comprehensive validation error types**
   - Extend CoreError with specific validation variants
   - Add structured validation context/details

2. **Design validation architecture**
   - Create `AccountValidator` struct with validation methods
   - Define validation levels: Input → Business Rules → Data Integrity
   - Add validation configuration options

### Phase 2: Core Input Validation (High Priority)  
1. **Account name validation**
   - Length constraints (1-100 chars)
   - Character restrictions (no special chars)
   - Uniqueness within parent account

2. **Currency and symbol validation**
   - ISO 4217 currency code validation
   - Investment symbol format validation
   - Field consistency checks

3. **Account type/subtype validation**
   - Valid combinations (Asset + Cash, not Asset + Salary)
   - Required field validation based on subtype

### Phase 3: Business Rule Validation (High Priority)
1. **Account hierarchy validation**
   - Valid parent-child type relationships
   - Circular reference prevention  
   - Path format and depth limits

2. **Investment field validation**
   - Symbol ↔ quantity consistency
   - Decimal precision constraints
   - Required fields for investment accounts

3. **Real estate field validation**
   - Address format validation
   - Date/price consistency checks

### Phase 4: Data Integrity Validation (Medium Priority)
1. **Referential integrity**
   - Parent account existence
   - User ID validation (when ownership added back)
   - Foreign key constraint verification

2. **Cross-field consistency**
   - Account state validation
   - Investment vs non-investment field conflicts

### Phase 5: Integration & Testing (High Priority)
1. **Integrate validation into AccountService**
   - Add validation calls to all CRUD methods
   - Implement validation bypass for migrations/testing

2. **Comprehensive test coverage**
   - Unit tests for all validation scenarios
   - Integration tests with database
   - Performance benchmarks for validation overhead

## Implementation Details

**Files to Create:**
- `crates/assets-core/src/validation/mod.rs`
- `crates/assets-core/src/validation/account_validator.rs` 
- `crates/assets-core/src/validation/errors.rs`

**Files to Modify:**
- `crates/assets-core/src/error.rs` (extend validation errors)
- `crates/assets-core/src/services/account_service.rs` (add validation calls)
- `crates/assets-core/src/lib.rs` (expose validation module)

**Success Criteria:**
- All AccountService methods validate input before database operations
- Clear, actionable error messages for validation failures
- <10ms validation overhead per operation
- 100% test coverage for validation scenarios
- Zero data integrity issues in production

**Why This Approach:**
- **Foundation-first**: Validation prevents bad data from entering system
- **Incremental**: Each phase builds on previous, allowing testing/refinement
- **Performance-conscious**: Separate validation levels allow optimization
- **Financial software standards**: Rigorous validation essential for money tracking

This plan addresses the critical gap in your system - currently AccountService trusts all input, which is dangerous for financial data.