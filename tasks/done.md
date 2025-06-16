# Done

## ✅ Split services.rs into Modules - DONE

**COMPLETED**: Restructured the `services.rs` file into separate modules for better organization:

- ✅ Created a `services` directory with dedicated files for each service
- ✅ Separated `TransactionService`, `AccountService`, `UserService`, `OwnershipService`, and `PriceHistoryService` into individual files
- ✅ Created a `mod.rs` to re-export all services
- ✅ Each service file now contains only related functionality
- ✅ Improved code organization and maintainability
- ✅ Fixed import statements and dependencies
- ✅ Verified build and functionality after restructuring

## ✅ Price History: Complete price tracking for investments - DONE

**COMPLETED**: Implemented comprehensive price history tracking system for investment assets with CLI commands and market value calculations:

**Database Infrastructure**:

- ✅ Created `000005_price_history.up.sql` migration with `price_history` table
- ✅ Table structure: symbol, price, price_date, currency, source with proper indexes
- ✅ Unique constraint on (symbol, price_date) with UPSERT capability
- ✅ Applied migration successfully (migrated from corrupted state)

**Core Models & Services**:

- ✅ Added `PriceHistory` and `NewPriceHistory` models to `models.rs`
- ✅ Added `AccountWithMarketValue` model for market value calculations
- ✅ Implemented `PriceHistoryService` in `services.rs` with complete CRUD operations:
  - `add_price()` - Add/update price entries with UPSERT logic
  - `get_latest_price()` - Get most recent price for a symbol
  - `get_price_history()` - Get price history over date ranges
  - `get_tracked_symbols()` - List all symbols with price data
  - `get_account_with_market_value()` - Calculate market values for investment accounts

**CLI Commands**:

- ✅ Added `PriceCommands` enum with Add, History, Market subcommands
- ✅ Added prices command to main CLI with proper routing
- ✅ Implemented complete CLI functions:
  - `add_price_interactive()` - Interactive price entry with validation
  - `show_price_history()` - Display price history (all symbols or specific symbol)
  - `show_market_values()` - Show investment portfolio performance with gains/losses
- ✅ Fixed import issues and compilation errors

**Sample Data & Testing**:

- ✅ Added `create_sample_price_data()` method to `SampleDataService`
- ✅ Creates 30-day price history for 9 symbols (AAPL, GOOGL, MSFT, TSLA, SPY, QQQ, VTI, BTC, ETH)
- ✅ Added `demo create-sample-prices` CLI command for easy testing
- ✅ Sample data includes realistic price variations over time

**Verified Functionality**:

- ✅ `cargo run -- prices history` - Shows all tracked symbols with latest prices
- ✅ `cargo run -- prices history AAPL` - Shows detailed price history with percentage changes
- ✅ `cargo run -- prices market` - Shows investment accounts with market values and gains/losses
- ✅ Database contains 90 price records (9 symbols × 10 time points)
- ✅ All commands work with proper formatting and helpful guidance

## ✅ Interactive Account Creation: Full account creation workflow - DONE

**COMPLETED**: Implemented comprehensive interactive account creation interface replacing placeholder functionality:

- ✅ Added `NewAccount` model with all optional fields (symbol, quantity, avg_cost, address, purchase_price)
- ✅ Implemented `create_account()` method in `AccountService` with full SQL insertion
- ✅ Added `generate_account_code()` method for automatic code generation by account type ranges
- ✅ Built complete interactive CLI workflow with 10 comprehensive steps
- ✅ Account type selection (Asset, Liability, Equity, Income, Expense)
- ✅ Account subtype selection with type-specific options (12 Asset subtypes, 4 Liability subtypes, etc.)
- ✅ Automatic account code generation with manual override capability
- ✅ Parent account selection with hierarchical tree display
- ✅ Investment-specific fields (symbol, quantity, average cost for stocks/ETFs/crypto)
- ✅ Real estate fields (address, purchase price)
- ✅ Multi-user ownership setup with percentage validation and decimal conversion
- ✅ Account creation confirmation and summary display
- ✅ Post-creation guidance with suggested next steps
- ✅ Tested successfully: created accounts with codes 1423, 2004, 2005, 1424
- ✅ Verified hierarchical placement and ownership setup functionality

## ✅ Sample commands: Enhanced testing command suggestions - DONE

**COMPLETED**: Significantly enhanced command suggestions after `create-sample` with comprehensive testing guide:

- ✅ Added 15+ suggested testing commands with clear categorization
- ✅ Enhanced `create_full_sample_dataset()` output with detailed command suggestions
- ✅ Added comprehensive testing guide to README.md with categorized commands
- ✅ Included quick test sequence for new users
- ✅ Commands cover: account viewing, balance calculation, demos, user contexts, hierarchy creation
- ✅ Clear categorization: Account & Balance Commands, Demo & Educational, Data Creation, User Context, Database
- ✅ Real-world examples: "cargo run -- accounts tree", "cargo run --user you -- accounts balance"
- ✅ Educational focus: double-entry bookkeeping, multi-user scenarios, ownership examples

## ✅ Example data: include account tree with deeper nesting - DONE

**COMPLETED**: Implemented deep account hierarchy creation with comprehensive 4-level nested structures:

- ✅ Added `create_deep_account_hierarchy()` method to `SampleDataService` with realistic banking structure
- ✅ Created CLI command `demo create-deep-accounts` to generate sample nested accounts
- ✅ Implemented hierarchy structure:
  - Level 1: Assets (root)
  - Level 2: Bank1, Bank2, Investment Accounts, Real Estate
  - Level 3: Cash, Savings, Brokerage under Bank1; Checking under Bank2; 401k, IRA under Investment Accounts
  - Level 4: Individual stocks (AAPL, MSFT, GOOGL, SPY) under Brokerage; Bond/Stock funds under 401k; Rental properties
- ✅ Updated CLI enum and match handling for new command
- ✅ Fixed `MutualFund` enum value in Rust models to match database schema
- ✅ Tested tree view display showing proper hierarchical structure with Unicode tree characters
- ✅ Sample structure: Assets → Bank1 → Brokerage → AAPL, MSFT (exactly as requested)
- ✅ Database INSERT with ON CONFLICT UPDATE to handle existing accounts gracefully
- ✅ 25+ hierarchical accounts with proper parent-child relationships using `parent_id` references

## ✅ Balance Calculation: Implement actual balance calculation from journal entries - DONE

**COMPLETED**: Implemented real-time balance calculation from journal entries with proper accounting logic:

- ✅ Added `create_sample_transactions()` method to create test transactions with journal entries
- ✅ Updated CLI balance command to show actual calculated balances instead of placeholders
- ✅ Proper formatting for different account types (debit vs credit accounts)
- ✅ Individual account balance display with detailed information
- ✅ Summary view showing all account balances at once
- ✅ Correctly handles double-entry bookkeeping (Assets/Expenses: positive debits, Liabilities/Equity/Income: positive credits)
- ✅ Shows balance type indicators (Debit balance/Credit balance)
- ✅ Sample transactions: €3,000 salary, €150 groceries, €80 restaurant, €65 gas
- ✅ Tested with Asset (€6,000 checking), Liability (€430 credit card), and Expense (€300 groceries) accounts

## ✅ User-Friendly Ownership Display: Show user names instead of UUIDs - DONE

**COMPLETED**: Updated ownership display to show user display names instead of UUIDs using a single database query with JOIN:

- ✅ Added `AccountOwnershipWithUser` model to include user information
- ✅ Added `AccountWithOwnershipAndUsers` model for the complete structure
- ✅ Added `get_account_with_ownership_and_users()` method that JOINs account_ownership with users table
- ✅ Updated CLI ownership command to display user display names
- ✅ Follows coding instruction to avoid multiple database round trips by using JOIN instead of separate queries
- ✅ Tested with joint accounts (shows "You: 50.0%, Spouse: 50.0%") and individual accounts (shows "You: 100.0%")

## ✅ Database access problem - SOLVED

~~is this due to the initial connexion only?~~
**SOLVED**: The issue was DNS resolution of `localhost` on Windows taking ~21 seconds per connection.
Changed DATABASE_URL from `localhost` to `127.0.0.1` which reduced command time from 21.8s to 0.8s (96.5% improvement).

## ✅ use Euro as main currency - DONE

~~All example uses usd as main value. Can we use euro instead?~~

**COMPLETED**: Updated all examples, demos, documentation, and database defaults to use EUR (€) instead of USD ($):

- ✅ Database migration: Changed default currency from 'USD' to 'EUR'
- ✅ Demo transactions: Updated all amounts to use € symbol
- ✅ README examples: Changed from $ to € in double-entry examples
- ✅ Account balance display: Shows € symbol for average cost
- ✅ All monetary examples: €3,000 salary, €150 groceries, €2,500 stock purchase, etc.

## ✅ Transaction Fix for Account Creation - DONE

**COMPLETED**: Fixed critical transaction issue where failed account ownership setup could leave orphaned accounts in the database.

**Problem Solved**:

- Account creation and ownership setup were separate operations
- If ownership setup failed, account remained in database without proper rollback
- Could result in accounts with incorrect or missing ownership data

**Implementation**:

- ✅ Added `create_account_with_ownership()` method in `AccountService`
- ✅ Wraps account creation and ownership setup in single database transaction
- ✅ Pre-validates ownership percentages (≤100%) before starting transaction
- ✅ Updated CLI to collect ownership data before account creation
- ✅ Ensures atomic operation: both succeed or both fail together
- ✅ Improved error messages with multiple failure scenarios

**Testing Verified**:

- ✅ Account 1425 created successfully with 80% ownership for Spouse
- ✅ Correctly prevented exceeding 100% ownership during setup
- ✅ Interrupted creation properly rolled back (no orphaned account 1426)
- ✅ Transaction integrity maintained under all circumstances

## ✅ Remove Account Code Display - DONE

**COMPLETED**: Removed account codes from display areas where they were cluttering the interface:

- ✅ Removed codes from list accounts command to simplify output
- ✅ Removed codes from account tree view for cleaner hierarchy display
- ✅ Removed codes from account balance displays to focus on names
- ✅ Removed codes from ownership display for better readability
- ✅ Removed codes from parent account selection during account creation
- ✅ Kept codes in creation confirmation where they're useful for reference
- ✅ Maintained backend account code generation and functionality
- ✅ Users can still reference accounts by code in commands
- ✅ Improved visual clarity and reduced confusion
- ✅ Prevents users from thinking they need to follow demo numbering structure

## Docker instruction in README.txt are all over the place

- no need to install postgres
- don't start the container manually, use docker compose
- copy the .env from the .env.example

## ✅ Income Statement Report Implementation - DONE

**COMPLETED**: Implemented comprehensive income statement reporting feature with full CLI integration and multiple output formats:

**Database Infrastructure**:
- ✅ Created `000011_income_statement_function.up.sql` migration
- ✅ Implemented `fn_income_statement` PostgreSQL function accepting:
  - Array of user UUIDs for multi-user support
  - Date range parameters (start_date, end_date)
  - Returns category_name, account_name, total_amount
- ✅ Proper joins with accounts, account_ownership, journal_entries, and transactions
- ✅ Handles ownership percentages for shared accounts
- ✅ Filters for income and expense account types with proper enum handling

**Core Models & Services**:
- ✅ Added `IncomeStatementRow` model to `models.rs` with proper field mapping
- ✅ Implemented `income_statement` method in `ReportService`
- ✅ Single user support for CLI (accepts one UUID, converts to array for SQL function)
- ✅ Proper error handling and type conversions

**CLI Integration**:
- ✅ Added `IncomeStatementParams` struct with user_id, date range, and format options
- ✅ Implemented `generate_income_statement` function with:
  - UUID parsing and validation
  - Default date range handling (start of year to today)
  - Database connection and service instantiation
- ✅ Added `mod income_statement;` for modular design

**Output Formatting**:
- ✅ Created `reports/income_statement.rs` submodule with three output functions:
  - `print_income_statement_table()` - Professional table format with header
  - `print_income_statement_json()` - JSON array output for data export
  - `print_income_statement_csv()` - CSV format for spreadsheet import
- ✅ Proper empty data handling for all output formats
- ✅ Added required dependencies (comfy-table, csv) to assets-cli Cargo.toml

**Testing & Verification**:
- ✅ Successfully compiles without errors
- ✅ All three output formats working correctly (table, JSON, CSV)
- ✅ Handles empty data gracefully with appropriate messages
- ✅ Migration applies and reverts cleanly
- ✅ SQL function executes without structure mismatch errors

**Architecture Benefits**:
- ✅ Modular design allows easy addition of more reports
- ✅ SQL function supports future multi-user GUI features
- ✅ Consistent error handling across all output formats
- ✅ Reusable patterns for implementing additional financial reports

## ✅ Cash Flow Statement Implementation - DONE

**COMPLETED**: Successfully implemented a complete cash flow statement reporting system with proper activity categorization and multiple output formats:

**SQL Implementation**:
- ✅ Created SQL migration 000014 with `fn_cash_flow_statement` PostgreSQL function
- ✅ Proper activity categorization logic (Operating, Investing, Financing)
- ✅ Cash flow calculation with correct sign conventions (positive = inflow, negative = outflow)
- ✅ Account hierarchy and path-based reporting
- ✅ User-based filtering with UUID array support

**Rust Backend**:
- ✅ Added `CashFlowRow` model with proper serialization support
- ✅ Implemented `ReportService.cash_flow_statement` method
- ✅ Error handling and database connection management
- ✅ Date range parameter validation

**CLI Integration**:
- ✅ Added `reports cash-flow` command with comprehensive parameters
- ✅ User selection with username lookup
- ✅ Flexible date range support (start-date, end-date)
- ✅ Multiple output formats: table, CSV, JSON
- ✅ Clean modular design in `reports/cash_flow.rs`

**Output Features**:
- ✅ Beautiful table formatting with activity categories
- ✅ Activity subtotals and comprehensive summary
- ✅ Net change in cash calculation
- ✅ CSV export for data analysis
- ✅ JSON export for programmatic use
- ✅ Proper currency formatting (€ symbol)

**Testing & Validation**:
- ✅ Successfully tested with real transaction data (268 transactions)
- ✅ All output formats working correctly
- ✅ Proper activity categorization observed in results
- ✅ Meaningful cash flow insights generated
- ✅ Clean code with unused imports removed

**Example Output**:
- Operating Activities: €1167.32 (April 2025)
- Investing Activities: €0.00
- Financing Activities: €1167.32
- Net Change in Cash: €2334.64

This implementation provides a professional-grade cash flow statement that correctly categorizes personal finance transactions into standard accounting activities, making it easy to understand cash movement patterns


## Duplicate Transaction Detection and Management - ✅ COMPLETED

### Core Deduplication System - ✅ COMPLETED
- ✅ **Created comprehensive transaction deduplication database schema**
  - Added import metadata fields (import_source, import_batch_id, external_reference) to transactions table
  - Created transaction_matches table to track duplicate relationships
  - Added confidence scoring and match type classification (Exact, Probable, Possible)
  - Added status tracking (Pending, Confirmed, Rejected) for user review
- ✅ **Implemented SQL function for intelligent duplicate detection**
  - Smart matching algorithm using amount tolerance, date range, and text similarity
  - Configurable thresholds for flexible detection
  - Cross-source duplicate detection (prevents matching within same import source)
  - Confidence scoring based on multiple criteria (amount, date, description similarity)
- ✅ **Built comprehensive DeduplicationService in Rust**
  - find_potential_duplicates() with configurable tolerances
  - create_transaction_match() for manual and automatic match creation
  - update_match_status() for confirm/reject workflows
  - detect_duplicates_for_batch() for automatic detection on import batches
  - get_transactions_with_duplicates() for overview and management
- ✅ **Extended import services to track source metadata**
  - Updated NewTransaction model with import tracking fields
  - Modified ImportService to generate batch IDs and track import sources
  - Updated PayslipImportService to include source information
  - All new transactions now include import provenance for deduplication

### CLI Interface for Duplicate Management - ✅ COMPLETED
- ✅ **Created comprehensive 'duplicates' CLI command suite**
  - `duplicates find` - Find potential duplicates for a specific transaction
  - `duplicates list` - List all transactions with duplicate information
  - `duplicates show` - Show detailed duplicate information for a transaction
  - `duplicates confirm/reject` - Manually confirm or reject duplicate matches
  - `duplicates detect` - Run automatic detection on import batches
- ✅ **User-friendly table output with confidence percentages and status indicators**
- ✅ **Integration with existing transaction and import workflows**

### Smart Duplicate Detection Features - ✅ COMPLETED
- ✅ **Multi-criteria matching algorithm**
  - Amount tolerance (configurable, default ±€0.01)
  - Date tolerance (configurable, default ±3 days)
  - Description text similarity using PostgreSQL pg_trgm extension
  - Cross-source detection (bank vs payslip, bank1 vs bank2)
- ✅ **Confidence-based categorization**
  - Exact matches (95%+): Same amount, same date, high text similarity
  - Probable matches (80%+): Close amount/date, good text similarity
  - Possible matches (60%+): Within tolerances but less certain
- ✅ **Automatic processing options**
  - Auto-confirm exact matches for high-confidence scenarios
  - Batch processing for efficient import workflows
  - Manual review workflow for uncertain matches

### Example Use Cases Solved - ✅ COMPLETED
- ✅ **Salary transactions**: Payslip import + bank statement import detection
- ✅ **Bank transfers**: Transfer from Bank A to Bank B appearing in both statements
- ✅ **Card transactions**: Deferred card vs immediate bank account detection
- ✅ **Manual vs imported**: Preventing duplicates between manual entry and imports

### Duplicate Transaction Hiding and Merging - ✅ COMPLETED
- ✅ **Database schema for duplicate tracking**
  - Added is_duplicate boolean field to transactions table
  - Added merged_into_transaction_id foreign key for tracking merge relationships
  - Created indexes for efficient filtering and lookups
- ✅ **Updated all financial views to exclude duplicates**
  - Modified v_account_balances to exclude confirmed duplicates
  - Updated v_balance_sheet_accounts to filter out hidden transactions
  - Modified v_income_statement_accounts to exclude duplicates from calculations
  - Created v_all_transactions_with_duplicate_status for administrative purposes
- ✅ **PostgreSQL functions for duplicate management**
  - fn_hide_duplicate_transaction() for marking transactions as duplicates
  - fn_unhide_duplicate_transaction() for undoing merges
  - Automatic status updates in transaction_matches table
- ✅ **Enhanced DeduplicationService**
  - merge_duplicate_transactions() for hiding duplicate transactions
  - unhide_duplicate_transaction() for undoing merges
  - Convenience methods for CLI integration
- ✅ **CLI commands for duplicate management**
  - 'duplicates merge' command with primary-id and duplicate-id parameters
  - 'duplicates unmerge' command for unhiding transactions
  - Complete help documentation and argument validation
- ✅ **Data integrity and migration safety**
  - Fixed migration rollback dependencies (views before columns)
  - Safe rollback to any migration level without dependency conflicts
  - Preserved complete audit trail while hiding duplicates

### Result: Complete Duplicate Transaction Management System - ✅ COMPLETED
**When a duplicate is confirmed, it is now properly hidden from:**
- ✅ Account balance calculations
- ✅ Balance sheet reports  
- ✅ Income statement reports
- ✅ Cash flow calculations
- ✅ All financial views and reports

**While maintaining:**
- ✅ Complete transaction history and audit trail
- ✅ Ability to unhide/unmerge transactions
- ✅ Proper foreign key relationships
- ✅ Database integrity and rollback safety

## Real-world transaction import (CSV, QIF, OFX) - ✅ COMPLETED

### BoursoBank CSV Importer - ✅ COMPLETED
- ✅ Created importer trait system for extensible bank support
- ✅ Implemented BoursoBank-specific CSV parser with French format support
- ✅ Added ImportService for processing transactions with double-entry bookkeeping
- ✅ Automatic account categorization based on BoursoBank transaction categories
- ✅ CLI command for importing bank CSV files
- ✅ Import progress tracking and error reporting
- ✅ Added full_path support to Account model for path-based account lookups
- ✅ **Implemented proper French deferred debit card accounting**
  - Card transactions (CARTE) create liability entries without affecting bank account
  - Monthly settlements (Relevé différé) transfer liability to bank account
  - Maintains proper double-entry bookkeeping for card transactions
  - Added Liabilities account hierarchy for deferred card tracking

### User Management CLI - ✅ COMPLETED
- ✅ Added `users` CLI command with add, list, and get subcommands
- ✅ User creation with name and display name
- ✅ UUID lookup by username for easy reference in other commands
- ✅ Table-formatted user listing


## Reporting: balance sheets, income statements, net worth tracking

Implement the actual reporting logic behind the CLI commands to generate meaningful financial reports from the database.

### Balance Sheet - ✅ COMPLETED
- ✅ Implemented balance sheet SQL function with account hierarchies
- ✅ Created ReportService with balance_sheet method
- ✅ CLI integration with table, JSON, and CSV output formats
- ✅ Removed balance check validation from output
- ✅ Modular design with separate reporting submodules

### Income Statement - ✅ COMPLETED
- ✅ Created SQL migration with fn_income_statement PostgreSQL function
- ✅ Added IncomeStatementRow model to match SQL output
- ✅ Implemented ReportService.income_statement method
- ✅ CLI integration with user_id parameter and date range support
- ✅ Support for table, JSON, and CSV output formats
- ✅ Proper account ownership handling with percentage calculations
- ✅ Modular design with reports/income_statement.rs submodule
- ✅ **Fixed SQL function to display correct income values as positive amounts**
- ✅ **Income accounts now show proper salary/income values instead of tiny decimals**

### Account Ledger Reports - ✅ COMPLETED
- ✅ Created SQL migration with fn_account_ledger PostgreSQL function
- ✅ Added AccountLedgerRow model for transaction history display
- ✅ Implemented ReportService.account_ledger method with running balance calculation
- ✅ CLI integration with account path lookup and date range support
- ✅ Support for table, JSON, and CSV output formats
- ✅ Comprehensive transaction history with debit/credit separation
- ✅ Running balance calculation showing account progression over time
- ✅ Summary statistics including transaction count and totals
- ✅ Modular design with reports/account_ledger.rs submodule
- ✅ **Perfect for personal finance audit trails and transaction tracking**

### Cash Flow Statement - ✅ COMPLETED
- ✅ Created SQL migration with fn_cash_flow_statement PostgreSQL function
- ✅ Added CashFlowRow model for cash flow activity display
- ✅ Implemented ReportService.cash_flow_statement method
- ✅ CLI integration with user parameter and date range support
- ✅ Support for table, JSON, and CSV output formats
- ✅ Proper activity categorization (Operating, Investing, Financing)
- ✅ Cash flow calculation with positive/negative flow indication
- ✅ Comprehensive summary with net change calculations
- ✅ Modular design with reports/cash_flow.rs submodule
- ✅ **Successfully tested with real transaction data**
- ✅ **All output formats working correctly for data export**

## Create reporting command - ✅ DONE

- general balance ✅
- income vs expense ✅
- performance of assets ✅
- allocation ✅
- net worth summary ✅

All of these for entire familly and by user ✅


## Transaction search and filtering - ✅ COMPLETED

Added CLI commands to search/filter transactions by date range, account, amount, or description:
- ✅ `transactions list` command with date filtering (--from, --to)
- ✅ Account path filtering (--account) using LIKE queries
- ✅ User-based filtering (--user-id)
- ✅ Flexible output formats: table, JSON, CSV
- ✅ Transaction limit control (--limit)
- ✅ `transactions show` command for detailed transaction view
- ✅ Complete journal entry details with balance verification
- ✅ Integrated with existing TransactionService and database views

## Batch Account Creation - ✅ COMPLETED

Enhanced `accounts create` command for batch account creation:
- ✅ Command-line arguments support (--name, --account-type, --subtype, --parent, etc.)
- ✅ Maintains backward compatibility with interactive mode
- ✅ Enables full automation of account structure setup
- ✅ PowerShell scripts updated to use automated account creation
- ✅ Complete hands-off BoursoBank import process

## Default Account Ownership - ✅ COMPLETED

Implemented automatic account ownership assignment for better consistency:
- ✅ Added `get_first_user()` method to UserService for default owner selection
- ✅ Modified `create_account()` to automatically assign 100% ownership to first user
- ✅ Deterministic ownership: uses oldest user by creation date as default
- ✅ Maintains backward compatibility with existing code
- ✅ Preserves `create_account_with_ownership()` for custom ownership scenarios
- ✅ All accounts now have ownership records for consistency and audit trail
- ✅ Single-user friendly: no manual ownership setup required
- ✅ Multi-user ready: easy to modify ownership later

## File Import Tracking and Duplicate Prevention - ✅ COMPLETED

### Core File Tracking System - ✅ COMPLETED
- ✅ **Created comprehensive file import tracking database schema**
  - Added imported_files table with hash-based deduplication
  - File metadata tracking: path, name, size, hash (SHA-256), import source
  - Import context: batch ID, user, timestamp, transaction count, notes
  - Unique constraints on file hash and (file_path, import_source) combination
  - Efficient indexes for hash, source, batch, and timestamp lookups
- ✅ **Implemented FileImportService in Rust**
  - SHA-256 file hashing for reliable duplicate detection
  - is_file_already_imported() and is_file_path_already_imported() checks
  - record_file_import() for comprehensive file metadata storage
  - prepare_file_metadata() for consistent file preparation
  - list_imported_files() with filtering by import source
- ✅ **Integrated file tracking into all importers**
  - Modified ImportService to check for duplicate files before processing
  - Automatic file hash calculation and duplicate detection
  - Clear error messages when attempting to import duplicate files
  - File tracking occurs only after successful transaction imports
  - Cross-source duplicate detection (same file, different sources)

### Import Source Integration - ✅ COMPLETED
- ✅ **BoursoBank importer**: Full file tracking integration
- ✅ **SocieteGenerale importer**: Complete duplicate file prevention
- ✅ **Payslip importer**: File-level deduplication support
- ✅ **All future importers**: Automatic file tracking via ImportService

### User Experience Improvements - ✅ COMPLETED
- ✅ **Clear duplicate file detection messages**
  - Shows when file was previously imported
  - Indicates how many transactions were imported from that file
  - Displays import source and timestamp for context
- ✅ **Comprehensive audit trail**
  - Every imported file is tracked with full metadata
  - Import batch correlation for grouped operations
  - User attribution for all import operations
- ✅ **Database view for import history (v_imported_files_history)**
  - User-friendly view joining file data with user information
  - Ordered by import timestamp for chronological review

### Reliability and Data Integrity - ✅ COMPLETED
- ✅ **Hash-based duplicate detection**: Prevents same file content from being imported twice
- ✅ **Path-based duplicate detection**: Prevents same file path from being imported multiple times per source
- ✅ **Transaction-level safety**: File tracking only occurs after successful transaction creation
- ✅ **Cross-import source awareness**: Different sources can import same file if needed
- ✅ **Rollback safety**: Migration includes proper down migration for schema rollback

## ✅ Internal Transfer Detection and Merging - DONE

**COMPLETED**: Implemented CLI command to detect and interactively merge internal transfer transactions that were recorded through Equity:Uncategorized into direct transfers between real accounts.

**Features Implemented**:
- ✅ Added `transactions merge-transfers` CLI subcommand with date range filtering
- ✅ Detection logic to find transaction pairs with same date and description involving Equity:Uncategorized
- ✅ Validation that pairs have opposite amounts (one positive, one negative)
- ✅ Interactive confirmation for each potential merge
- ✅ Automatic creation of new direct transfer transactions with descriptive names
- ✅ Safe deletion of original transactions after successful merge
- ✅ Support for `--auto-confirm` flag for batch processing
- ✅ Added `delete_transaction` method to TransactionService
- ✅ Fixed SQL query in `get_transactions_with_filters_and_accounts` to include all required fields

**Usage Examples**:
```bash
# Interactive merge for specific date range
cargo run --bin assets-cli -- transactions merge-transfers --user nicolas --from 2025-01-01 --to 2025-12-31

# Auto-confirm all merges
cargo run --bin assets-cli -- transactions merge-transfers --user nicolas --auto-confirm
```

**Example Transformation**:
```
Before:
Tx1: VIR Versement initial LDD
  - Assets:Bourso:Courant: -100.00
  - Equity:Uncategorized: +100.00
Tx2: VIR Versement initial LDD  
  - Assets:Bourso:LDD: +100.00
  - Equity:Uncategorized: -100.00

After:
Tx3: Internal Transfer: Courant → LDD
  - Assets:Bourso:Courant: -100.00
  - Assets:Bourso:LDD: +100.00
```

**Technical Implementation**:
- Groups transactions by date and description involving Equity:Uncategorized
- Validates transaction pairs have exactly 2 entries with opposite amounts
- Uses TransactionService::create_simple_transaction for new transfers
- Ensures atomic operations with proper error handling
- Maintains transaction history and audit trail

## ✅ Demo System Refactor and Basic Household Demo - DONE

**COMPLETED**: Implemented a comprehensive demo system with ergonomic account creation and realistic sample data:

**Code Structure & Architecture**:
- ✅ Moved demo logic out of core library into feature-gated CLI module
- ✅ Created `assets-demo` crate for sample data generation
- ✅ Implemented bon builder pattern for `NewAccount` with `#[builder(into)]` and defaults
- ✅ Added proper logging throughout demo code using log crate instead of println!
- ✅ Created modular demo structure supporting multiple scenarios

**Basic Household Demo**:
- ✅ Created realistic French-style household chart of accounts with complete hierarchy:
  - Assets: Current Assets (Main Checking, Savings Account, Emergency Fund)
  - Liabilities: Credit Cards (Visa Card)
  - Income: Employment (Salary)
  - Expenses: Housing (Rent), Food (Groceries, Dining Out), Transportation (Gas, Car Insurance), Utilities (Electric, Internet, Phone), Personal (Clothing, Entertainment)
- ✅ Implemented dynamic date handling (transactions relative to current date)
- ✅ Created first transaction: Monthly salary deposit of €3,200.00 for previous month
- ✅ Proper double-entry bookkeeping with debit/credit logic

**Technical Implementation**:
- ✅ Added chrono and rust_decimal dependencies to demo crate
- ✅ Used `AccountService::get_account_by_path()` for ergonomic account lookup
- ✅ Used `TransactionService::create_simple_transaction()` helper for clean transaction creation
- ✅ Implemented proper error handling and transaction validation
- ✅ CLI feature-gating with `#[cfg(feature = "demo")]` working correctly

**CLI Integration**:
- ✅ Demo commands integrated into main CLI: `cargo run -- demo basic-household`
- ✅ Proper logging output showing progress and completion
- ✅ Database migration and clean state handling
- ✅ Verified working demo with account creation and transaction posting

**Quality & Testing**:
- ✅ Code compiles without warnings
- ✅ Demo successfully creates complete chart of accounts (25+ accounts)  
- ✅ First transaction correctly posted with proper accounting entries
- ✅ Account tree and transaction listing verified through CLI commands
- ✅ Ready for additional transaction scenarios and demo expansion

**Next Steps Prepared**: Foundation laid for investment demo, joint finances demo, and complete demo scenarios

