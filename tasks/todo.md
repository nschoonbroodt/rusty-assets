# Actual Todo


## Review balance sheet account sign conventions

The balance sheet is showing some asset accounts (BoursoBank, SG) with negative balances, which doesn't follow standard accounting conventions. Need to review:
- Whether transaction posting logic correctly handles debits/credits for different account types
- If balance sheet display should flip signs for proper presentation 
- Ensure consistency between balance sheet and income statement sign conventions

## Automated Price Feeds

Implement automated price feeds from financial APIs for real-time price updates instead of manual entry. This would allow the system to automatically fetch current market prices for tracked assets (stocks, ETFs, crypto) from sources like Yahoo Finance, Alpha Vantage, or similar APIs.

## Add github actions

## Future Import Extensions
- [ ] Generic CSV importer for other banks
- [ ] QIF format support
- [ ] OFX format support


## Web or GUI interface

Develop a web or graphical user interface as an alternative to the CLI for broader accessibility and ease of use.

## Reporting: balance sheets, income statements, net worth tracking

Implement the actual reporting logic behind the CLI commands to generate meaningful financial reports from the database.

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


CLI structure with clap is complete, need to implement the actual reporting logic.

## import bank statement

- csv or other
- automatic spending classification (based on rule, machine learning?)

## Error handling improvements

Add proper error handling and user-friendly error messages throughout the CLI, especially for database operations and user input validation.

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

## Review balance sheet account sign conventions

The balance sheet is showing some asset accounts (BoursoBank, SG) with negative balances, which doesn't follow standard accounting conventions. Need to review:
- Whether transaction posting logic correctly handles debits/credits for different account types
- If balance sheet display should flip signs for proper presentation 
- Ensure consistency between balance sheet and income statement sign conventions

## Automated Price Feeds

Implement automated price feeds from financial APIs for real-time price updates instead of manual entry. This would allow the system to automatically fetch current market prices for tracked assets (stocks, ETFs, crypto) from sources like Yahoo Finance, Alpha Vantage, or similar APIs.

## Add github actions

## Real-world transaction import (CSV, QIF, OFX) - ✅ COMPLETED

### Future Import Extensions
- [ ] Generic CSV importer for other banks
- [ ] QIF format support
- [ ] OFX format support


## Web or GUI interface

Develop a web or graphical user interface as an alternative to the CLI for broader accessibility and ease of use.

## Reporting: balance sheets, income statements, net worth tracking

Implement the actual reporting logic behind the CLI commands to generate meaningful financial reports from the database.

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


CLI structure with clap is complete, need to implement the actual reporting logic.

## import bank statement

- csv or other
- automatic spending classification (based on rule, machine learning?)

## Error handling improvements

Add proper error handling and user-friendly error messages throughout the CLI, especially for database operations and user input validation.

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

