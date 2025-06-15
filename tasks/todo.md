# High Priority Issues

## Balancing with units
Check how we manage transactions. I think currently we only have an amount, no units?

## Review balance sheet account sign conventions
The balance sheet is showing some asset accounts (BoursoBank, SG) with negative balances, which doesn't follow standard accounting conventions. Need to review:
- Whether transaction posting logic correctly handles debits/credits for different account types
- If balance sheet display should flip signs for proper presentation 
- Ensure consistency between balance sheet and income statement sign conventions

# Core Features

## Import & Data Processing
- [ ] Generic CSV importer for other banks
- [ ] QIF format support
- [ ] OFX format support
- [ ] Automatic spending classification (based on rules, machine learning?)

## Reporting & Analytics
- [ ] Net worth tracking over time
- [ ] Trial balance report
- [ ] CSV/JSON export for reports
- [ ] Date range validation and defaults (current month, year, YTD, etc.)

### Reporting Database Views
Create SQL views for common reporting queries to optimize performance and avoid code duplication:
- Balance sheet view with account hierarchies
- Income statement view with revenue/expense categorization
- Cash flow view with operating/investing/financing activities
- Trial balance view with current balances by account

### Cash Flow Improvements (Lower Priority)
- [ ] Refine account categorization (e.g., large "Pending" transfers)
- [ ] Better investment activity detection
- [ ] Add beginning/ending cash balance display
- [ ] Enhanced category mapping for more accurate activity classification

## Price & Market Data
- [ ] Automated price feeds from financial APIs (Yahoo Finance, Alpha Vantage)
- [ ] Real-time price updates for stocks, ETFs, crypto

## System Improvements
- [ ] Error handling improvements with user-friendly messages
- [ ] Backup and restore functionality
- [ ] Account archiving/deactivation for closed accounts
- [ ] Add GitHub Actions for CI/CD

# User Interface

## Terminal Interface
- [ ] Create a terminal user interface with ratatui

## Web/GUI Interface
- [ ] Web or GUI interface for broader accessibility
- [ ] UI using a Rust framework (egui, iced, etc.)
- [ ] UI using Tauri for desktop apps

# Security & Multi-user
- [ ] Database encryption
- [ ] Authentication system
- [ ] Row-based access control

# Future Ideas (Long Term)

## Advanced Features
- [ ] Budget goal tracking
- [ ] Automatic loan prediction
- [ ] Future tax estimation
- [ ] Multi "main currency" support

## Platform Extensions
- [ ] Web API
- [ ] Mobile app
- [ ] Local web app

---

**Note**: CLI structure with clap is complete, need to implement the actual reporting logic behind the commands.

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

