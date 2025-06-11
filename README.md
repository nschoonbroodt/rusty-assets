# RustyAssets

RustyAssets is a modular, extensible personal finance tracker written in Rust. It helps you track:

- **Bank accounts**
- **Spending by category**
- **Investments** (ETFs, shares, etc.)
- **Real estate** (property value and mortgage tracking)
- **Other assets** (easily extensible)

## Project Structure

This project is a Cargo workspace with all crates in the `crates/` directory:

- `assets-core`: Core logic, computation, and PostgreSQL database models/connection.
- `assets-cli`: Command-line interface for interacting with your data.

## Features

- Modular design for easy extension (e.g., add GUI/web interface as new crates)
- PostgreSQL for persistent, reliable storage
- Designed for future features like:
  - Recurring transactions
  - Budgeting
  - Net worth calculation
  - Multi-currency support
  - Reporting and analytics

## Getting Started

1. Install [Rust](https://www.rust-lang.org/tools/install) and [PostgreSQL](https://www.postgresql.org/download/).
2. Clone this repository.
3. Build the workspace:
   ```powershell
   cargo build --workspace
   ```
4. Run the CLI:
   ```powershell
   cargo run -p assets-cli
   ```

## Using PostgreSQL with Docker

The easiest way to run a local PostgreSQL instance is with Docker:

```powershell
docker run --name rustyassets-postgres -e POSTGRES_PASSWORD=rustyassets -e POSTGRES_USER=rustyassets -e POSTGRES_DB=rustyassets -p 5432:5432 -d postgres:16
```

- This will start a PostgreSQL 16 container with user, password, and database all set to `rustyassets`.
- You can change these values as needed.
- To stop and remove the container:
  ```powershell
  docker stop rustyassets-postgres
  docker rm rustyassets-postgres
  ```

Update your `.env` or configuration files to match these credentials for local development.

## Development

### Running Migrations

When you're ready to connect to a database:

1. Start PostgreSQL with Docker:

   ```powershell
   docker run --name rustyassets-postgres -e POSTGRES_PASSWORD=rustyassets -e POSTGRES_USER=rustyassets -e POSTGRES_DB=rustyassets -p 5432:5432 -d postgres:16
   ```

2. Set your database URL:

   ```powershell
   $env:DATABASE_URL="postgresql://rustyassets:rustyassets@localhost:5432/rustyassets"
   ```

3. Run migrations:

   ```powershell
   # Install SQLx CLI first
   cargo install sqlx-cli --no-default-features --features rustls,postgres

   # Run migrations
   sqlx migrate run --source crates/assets-core/migrations
   ```

### Project Status

✅ **Completed:**

- Double-entry bookkeeping system design
- Account types: Assets, Liabilities, Equity, Income, Expenses
- Transaction and journal entry models
- Database schema with PostgreSQL migrations
- Core business logic services
- CLI demo with interactive examples
- Rust workspace structure with proper separation

🔄 **Next Steps:**

- ✅ Database connection and migration runner
- CRUD operations for accounts and transactions
- Real-world transaction import (CSV, QIF, OFX)
- Investment price tracking and portfolio valuation
- Web or GUI interface
- Reporting: balance sheets, income statements, net worth tracking

## Database Setup

RustyAssets uses PostgreSQL for data persistence. You can run it locally with Docker:

### Quick Start with Docker

```powershell
# Start PostgreSQL in Docker
docker-compose up -d

# Initialize database and run migrations
cargo run -- db init

# Create sample data
cargo run -- demo create-sample
```

## Testing Guide

After setting up the database and creating sample data, explore RustyAssets with these commands:

### 🧪 Essential Testing Commands

```powershell
# 📊 View Your Data
cargo run -- accounts tree              # Beautiful hierarchical chart of accounts
cargo run -- accounts balance           # All account balances from real transactions
cargo run -- demo create-deep-accounts  # Create realistic 4-level account hierarchy

# 🎭 Learn Double-Entry Bookkeeping
cargo run -- demo double-entry          # Interactive examples with €3,000 salary, €150 groceries
cargo run -- demo account-types         # Understand debit/credit behavior by account type

# 👥 Multi-User Finance
cargo run -- demo multi-user            # Learn about shared ownership scenarios
cargo run -- demo ownership             # See fractional ownership examples
cargo run -- accounts ownership 1001    # Show ownership details for joint checking account

# 🗂️ Categories & Organization
cargo run -- demo categories            # Unlimited nesting category examples
cargo run -- demo create-deep-categories # Create Expense→Home→Deco→Furniture→Sofa

# 👤 User Context Switching
cargo run --user you -- accounts balance    # Your perspective only
cargo run --user spouse -- accounts balance # Spouse's perspective only  
cargo run --user family -- accounts balance # Combined family view (default)
```

### 🎯 Quick Test Sequence

```powershell
# 1. Create deep hierarchical data
cargo run -- demo create-deep-accounts

# 2. View the beautiful tree structure
cargo run -- accounts tree

# 3. See real balances calculated from transactions
cargo run -- accounts balance

# 4. Learn the accounting principles
cargo run -- demo double-entry

# 5. Try different user perspectives
cargo run --user you -- accounts balance
```

### Manual PostgreSQL Setup

If you prefer to install PostgreSQL manually:

```powershell
# Install PostgreSQL and create database
createdb rustyassets

# Update .env with your connection details
# DATABASE_URL=postgresql://username:password@localhost:5432/rustyassets

# Run migrations
cargo run --bin assets-cli -- init-db
```

### Database Commands

```powershell
# Initialize database and run migrations
cargo run --bin assets-cli -- init-db

# Check connection and migration status
cargo run --bin assets-cli -- db-status

# Create sample users and accounts
cargo run --bin assets-cli -- create-sample

# View multi-user examples
cargo run --bin assets-cli -- multi-user

# View fractional ownership examples
cargo run --bin assets-cli -- ownership
```

## Architecture

RustyAssets uses **double-entry bookkeeping**, the same accounting system used by businesses and professional financial software. This provides:

- **Complete audit trail** - see exactly where every euro comes from and goes
- **Built-in validation** - all transactions must balance (debits = credits)
- **Professional reporting** - can generate balance sheets, income statements, etc.
- **Unified account system** - cash, investments, real estate are all accounts

### Double-Entry Examples

**Getting paid €3,000 salary:**

```
Debit:  Checking Account (Asset)     +€3,000
Credit: Salary Income               +€3,000
```

**Buying €150 groceries with credit card:**

```
Debit:  Groceries Expense           +€150
Credit: Credit Card (Liability)     +€150
```

**Purchasing €2,500 of AAPL stock:**

```
Debit:  AAPL Stock (Asset)          +€2,500
Credit: Checking Account (Asset)    -€2,500
```

## Quick Demo

Try the built-in demo to see double-entry bookkeeping in action:

```powershell
# See interactive examples
cargo run -p assets-cli -- demo

# Learn about account types
cargo run -p assets-cli -- account-types

# Multi-user and ownership examples
cargo run -p assets-cli -- multi-user
cargo run -p assets-cli -- ownership

# Try user context switching
cargo run -p assets-cli -- --user you demo
cargo run -p assets-cli -- --user spouse demo
cargo run -p assets-cli -- --user family demo

# See all commands
cargo run -p assets-cli -- --help
```

## Contributing

Contributions and suggestions are welcome! See the project structure and add new crates for additional interfaces or features.

---

**Project name inspiration:**

- "RustyAssets" combines the app's goal (asset tracking) and the Rust language.
- Crate names are prefixed with `assets-` to avoid conflicts and clarify their purpose.
