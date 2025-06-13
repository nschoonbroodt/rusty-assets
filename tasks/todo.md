# Actual Todo

## Check the data model in sql.
I feel that there is something wrong with the category including income, assets, ...

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

### Remaining Reports
- [ ] Net worth tracking over time
- [ ] Cash flow statement
- [ ] Trial balance report
- [ ] Account ledger reports

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

## Transaction search and filtering

Add commands to search/filter transactions by date range, account, amount, or description to make the system more useful with real data.

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
