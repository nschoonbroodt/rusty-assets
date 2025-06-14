# Actual Todo

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

## Review balance sheet account sign conventions

The balance sheet is showing some asset accounts (BoursoBank, SG) with negative balances, which doesn't follow standard accounting conventions. Need to review:
- Whether transaction posting logic correctly handles debits/credits for different account types
- If balance sheet display should flip signs for proper presentation 
- Ensure consistency between balance sheet and income statement sign conventions

## Automated Price Feeds

Implement automated price feeds from financial APIs for real-time price updates instead of manual entry. This would allow the system to automatically fetch current market prices for tracked assets (stocks, ETFs, crypto) from sources like Yahoo Finance, Alpha Vantage, or similar APIs.

## Add github actions

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

### Future Import Extensions
- [ ] Generic CSV importer for other banks
- [ ] QIF format support
- [ ] OFX format support

### User Management CLI - ✅ COMPLETED
- ✅ Added `users` CLI command with add, list, and get subcommands
- ✅ User creation with name and display name
- ✅ UUID lookup by username for easy reference in other commands
- ✅ Table-formatted user listing

## Web or GUI interface

Develop a web or graphical user interface as an alternative to the CLI for broader accessibility and ease of use.

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

### Remaining Reports
- [ ] Net worth tracking over time
- [ ] Trial balance report

### Potential Cash Flow Improvements (Lower Priority)
- [ ] Refine account categorization (e.g., large "Pending" transfers)
- [ ] Better investment activity detection
- [ ] Add beginning/ending cash balance display
- [ ] Enhanced category mapping for more accurate activity classification

## Implement reporting database views

Create SQL views for common reporting queries to optimize performance and avoid code duplication:

- Balance sheet view with account hierarchies
- Income statement view with revenue/expense categorization
- Cash flow view with operating/investing/financing activities
- Trial balance view with current balances by account

## Add CSV/JSON export for reports

Extend the reporting commands to actually export data in CSV and JSON formats as specified in the CLI parameters.

## Add date range validation and defaults

Implement proper date handling in reports:

- Default date ranges (current month, year, etc.)
- Validation of start/end date relationships
- Support for relative dates (last month, YTD, etc.)

## Create reporting command - ✅ DONE

- general balance ✅
- income vs expense ✅
- performance of assets ✅
- allocation ✅
- net worth summary ✅

All of these for entire familly and by user ✅

CLI structure with clap is complete, need to implement the actual reporting logic.

## import bank statement

- csv or other
- automatic spending classification (based on rule, machine learning?)

## Error handling improvements

Add proper error handling and user-friendly error messages throughout the CLI, especially for database operations and user input validation.

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

## Backup and restore functionality

Add commands to export/import the entire database for backup purposes, especially important for personal finance data.

## Account archiving/deactivation

Add ability to archive old accounts without deleting historical data (useful for closed bank accounts, sold investments, etc.).

# Medium term

## Create a terminal user interface with ratatui

## Security

Does the database needs to be crypted? Do we need auth? Row based access?

## Auto update prices

such as share prices, crypto

# Long term - to be sorted

## UI using a rust framework

## UI using tauri

## possible web api

## possible mobile app

## possible web app (but local)

## budget goal tracking

## automatic loan prediction?

## future tax estimation

## multi "main currency" support?

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

## Review balance sheet account sign conventions

The balance sheet is showing some asset accounts (BoursoBank, SG) with negative balances, which doesn't follow standard accounting conventions. Need to review:
- Whether transaction posting logic correctly handles debits/credits for different account types
- If balance sheet display should flip signs for proper presentation 
- Ensure consistency between balance sheet and income statement sign conventions

## Automated Price Feeds

Implement automated price feeds from financial APIs for real-time price updates instead of manual entry. This would allow the system to automatically fetch current market prices for tracked assets (stocks, ETFs, crypto) from sources like Yahoo Finance, Alpha Vantage, or similar APIs.

## Add github actions

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

### Future Import Extensions
- [ ] Generic CSV importer for other banks
- [ ] QIF format support
- [ ] OFX format support

### User Management CLI - ✅ COMPLETED
- ✅ Added `users` CLI command with add, list, and get subcommands
- ✅ User creation with name and display name
- ✅ UUID lookup by username for easy reference in other commands
- ✅ Table-formatted user listing

## Web or GUI interface

Develop a web or graphical user interface as an alternative to the CLI for broader accessibility and ease of use.

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

### Remaining Reports
- [ ] Net worth tracking over time
- [ ] Trial balance report

### Potential Cash Flow Improvements (Lower Priority)
- [ ] Refine account categorization (e.g., large "Pending" transfers)
- [ ] Better investment activity detection
- [ ] Add beginning/ending cash balance display
- [ ] Enhanced category mapping for more accurate activity classification

## Implement reporting database views

Create SQL views for common reporting queries to optimize performance and avoid code duplication:

- Balance sheet view with account hierarchies
- Income statement view with revenue/expense categorization
- Cash flow view with operating/investing/financing activities
- Trial balance view with current balances by account

## Add CSV/JSON export for reports

Extend the reporting commands to actually export data in CSV and JSON formats as specified in the CLI parameters.

## Add date range validation and defaults

Implement proper date handling in reports:

- Default date ranges (current month, year, etc.)
- Validation of start/end date relationships
- Support for relative dates (last month, YTD, etc.)

## Create reporting command - ✅ DONE

- general balance ✅
- income vs expense ✅
- performance of assets ✅
- allocation ✅
- net worth summary ✅

All of these for entire familly and by user ✅

CLI structure with clap is complete, need to implement the actual reporting logic.

## import bank statement

- csv or other
- automatic spending classification (based on rule, machine learning?)

## Error handling improvements

Add proper error handling and user-friendly error messages throughout the CLI, especially for database operations and user input validation.

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

## Backup and restore functionality

Add commands to export/import the entire database for backup purposes, especially important for personal finance data.

## Account archiving/deactivation

Add ability to archive old accounts without deleting historical data (useful for closed bank accounts, sold investments, etc.).

# Medium term

## Create a terminal user interface with ratatui

## Security

Does the database needs to be crypted? Do we need auth? Row based access?

## Auto update prices

such as share prices, crypto

# Long term - to be sorted

## UI using a rust framework

## UI using tauri

## possible web api

## possible mobile app

## possible web app (but local)

## budget goal tracking

## automatic loan prediction?

## future tax estimation

## multi "main currency" support?

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

## Review balance sheet account sign conventions

The balance sheet is showing some asset accounts (BoursoBank, SG) with negative balances, which doesn't follow standard accounting conventions. Need to review:
- Whether transaction posting logic correctly handles debits/credits for different account types
- If balance sheet display should flip signs for proper presentation 
- Ensure consistency between balance sheet and income statement sign conventions

## Automated Price Feeds

Implement automated price feeds from financial APIs for real-time price updates instead of manual entry. This would allow the system to automatically fetch current market prices for tracked assets (stocks, ETFs, crypto) from sources like Yahoo Finance, Alpha Vantage, or similar APIs.

## Add github actions

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

### Future Import Extensions
- [ ] Generic CSV importer for other banks
- [ ] QIF format support
- [ ] OFX format support

### User Management CLI - ✅ COMPLETED
- ✅ Added `users` CLI command with add, list, and get subcommands
- ✅ User creation with name and display name
- ✅ UUID lookup by username for easy reference in other commands
- ✅ Table-formatted user listing

## Web or GUI interface

Develop a web or graphical user interface as an alternative to the CLI for broader accessibility and ease of use.

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

### Remaining Reports
- [ ] Net worth tracking over time
- [ ] Trial balance report

### Potential Cash Flow Improvements (Lower Priority)
- [ ] Refine account categorization (e.g., large "Pending" transfers)
- [ ] Better investment activity detection
- [ ] Add beginning/ending cash balance display
- [ ] Enhanced category mapping for more accurate activity classification

## Implement reporting database views

Create SQL views for common reporting queries to optimize performance and avoid code duplication:

- Balance sheet view with account hierarchies
- Income statement view with revenue/expense categorization
- Cash flow view with operating/investing/financing activities
- Trial balance view with current balances by account

## Add CSV/JSON export for reports

Extend the reporting commands to actually export data in CSV and JSON formats as specified in the CLI parameters.

## Add date range validation and defaults

Implement proper date handling in reports:

- Default date ranges (current month, year, etc.)
- Validation of start/end date relationships
- Support for relative dates (last month, YTD, etc.)

## Create reporting command - ✅ DONE

- general balance ✅
- income vs expense ✅
- performance of assets ✅
- allocation ✅
- net worth summary ✅

All of these for entire familly and by user ✅

CLI structure with clap is complete, need to implement the actual reporting logic.

## import bank statement

- csv or other
- automatic spending classification (based on rule, machine learning?)

## Error handling improvements

Add proper error handling and user-friendly error messages throughout the CLI, especially for database operations and user input validation.

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

## Backup and restore functionality

Add commands to export/import the entire database for backup purposes, especially important for personal finance data.

## Account archiving/deactivation

Add ability to archive old accounts without deleting historical data (useful for closed bank accounts, sold investments, etc.).

# Medium term

## Create a terminal user interface with ratatui

## Security

Does the database needs to be crypted? Do we need auth? Row based access?

## Auto update prices

such as share prices, crypto

# Long term - to be sorted

## UI using a rust framework

## UI using tauri

## possible web api

## possible mobile app

## possible web app (but local)

## budget goal tracking

## automatic loan prediction?

## future tax estimation

## multi "main currency" support?

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
