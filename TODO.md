# Actual Todo

## Automated Price Feeds

Implement automated price feeds from financial APIs for real-time price updates instead of manual entry. This would allow the system to automatically fetch current market prices for tracked assets (stocks, ETFs, crypto) from sources like Yahoo Finance, Alpha Vantage, or similar APIs.

## Add github actions

## Real-world transaction import (CSV, QIF, OFX)

Support importing transactions from common financial data formats (CSV, QIF, OFX) to make onboarding and data migration easier.

## Web or GUI interface

Develop a web or graphical user interface as an alternative to the CLI for broader accessibility and ease of use.

## Reporting: balance sheets, income statements, net worth tracking

Implement reporting features to generate balance sheets, income statements, and net worth tracking for comprehensive financial analysis.

## Docker instruction in README.txt are all over the place
- no need to install postgres
- don't start the container manually, use docker compose
- copy the .env from the .env.example


## Create reporting command

- general balance
- income vs expense
- performance of assets
- allocation
- net worth summary


All of these for entire familly and by user


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