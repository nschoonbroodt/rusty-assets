# RustyAssets

[![codecov](https://codecov.io/github/nschoonbroodt/rusty-assets/graph/badge.svg?token=HQ6I7ZDIT2)](https://codecov.io/github/nschoonbroodt/rusty-assets)

RustyAssets is a modular, extensible personal finance tracker written in Rust. It helps you track:

- **Bank accounts**
- **Spending by category**
- **Investments** (ETFs, shares, etc.)
- **Real estate** (property value and mortgage tracking)
- **Other assets** (easily extensible)

Note: This project was heavily help by LLMs (copilot & claude), so some parts are difficult to read, and use at your own risk.

## Project Structure

This project is a Cargo workspace with all crates in the `crates/` directory:

- `assets-core`: Core logic, computation, and PostgreSQL database models/connection.
- `assets-cli`: Command-line interface for interacting with your data.

## Features

- **Double-entry bookkeeping** with transaction validation
- **Multi-user support** with fractional ownership percentages
- **Hierarchical account structure** with unlimited nesting
- **Investment tracking** with price history and market value calculations
- **Interactive account creation** with type-specific fields
- **Real-time balance calculation** from journal entries
- **PostgreSQL** for persistent, reliable storage
- **EUR (€) as default currency** for all transactions and accounts
- **Database performance optimization** (uses 127.0.0.1 instead of localhost)
- **Modular design** for easy extension (e.g., add GUI/web interface as new crates)

### Planned Features

- Recurring transactions
- Budgeting
- Net worth calculation
- Real-world transaction import (CSV, QIF, OFX)
- Automated price feeds from financial APIs
- Reporting and analytics
- Web or GUI interface

## Getting Started with Docker (Recommended)

This is the recommended way to get RustyAssets up and running quickly.

1.  **Prerequisites**:

    - Install [Rust](https://www.rust-lang.org/tools/install).
    - Install [Docker Desktop](https://www.docker.com/products/docker-desktop/) (which includes Docker Compose).

2.  **Clone the Repository**:

    ```powershell
    git clone https://github.com/nschoonbroodt/rusty-assets/
    cd rustyassets
    ```

3.  **Configure Environment**:

    - Copy the example environment file:
      ```powershell
      Copy-Item .env.example .env
      ```
    - Open the newly created `.env` file and verify the `DATABASE_URL`. It should match the credentials in `docker-compose.yml` (default: `postgresql://rustyassets:rustyassets@127.0.0.1:5432/rustyassets`).
      > **Performance Tip**: Using `127.0.0.1` instead of `localhost` for the database host can significantly improve connection speed on Windows.

4.  **Start PostgreSQL using Docker Compose**:

    ```powershell
    docker-compose up -d
    ```

    This command will:

    - Download the `postgres:15` image if you don't have it.
    - Start a PostgreSQL container named `rustyassets-postgres`.
    - Create a user `rustyassets` with password `rustyassets`.
    - Create a database named `rustyassets`.
    - Persist data in a Docker volume named `rustyassets_postgres_data`.

5.  **Run Database Migrations**:

    - Install SQLx CLI (if you haven't already):
      ```powershell
      cargo install sqlx-cli --no-default-features --features rustls,postgres
      ```
    - Run the migrations:
      ```powershell
      sqlx migrate run --source crates/assets-core/migrations
      ```
      _(Ensure your `DATABASE_URL` in `.env` is correctly set for this step)._
      Alternatively, you can use the built-in CLI command (if available and configured to use the .env file):
      ```powershell
      cargo run -- db init
      ```

6.  **Build the Workspace**:

    ```powershell
    cargo build --workspace
    ```

7.  **Run the CLI**:

    ```powershell
    cargo run -p assets-cli -- --help
    ```

    This will show you the available commands.

8.  **(Optional) Create Sample Data**:
    ```powershell
    cargo run -- demo create-sample
    ```

### Managing the Docker Container

- **Stop the PostgreSQL container**:
  ```powershell
  docker-compose down
  ```
- **Stop and remove the data volume** (if you want a fresh start):
  ```powershell
  docker-compose down -v
  ```

## Alternative: Manual PostgreSQL Setup

If you prefer not to use Docker or have an existing PostgreSQL instance:

1.  **Install PostgreSQL**: Follow the official instructions for your OS from [postgresql.org](https://www.postgresql.org/download/).
2.  **Create Database and User**:
    - Create a database (e.g., `rustyassets`).
    - Create a user (e.g., `rustyassets`) with a password and grant necessary permissions to the database.
3.  **Configure Environment**:
    - Copy `.env.example` to `.env`.
    - Update `DATABASE_URL` in `.env` with your PostgreSQL connection string (e.g., `postgresql://youruser:yourpassword@yourhost:yourport/yourdatabase`).
4.  **Follow steps 5-8** from the "Getting Started with Docker" section above (Run Migrations, Build, Run CLI, Sample Data).

## Development

### Running Migrations

When you're ready to connect to a database:

1. Start PostgreSQL with Docker:

   ```powershell
   docker run --name rustyassets-postgres -e POSTGRES_PASSWORD=rustyassets -e POSTGRES_USER=rustyassets -e POSTGRES_DB=rustyassets -p 5432:5432 -d postgres:16
   ```

2. Set your database URL:

   ```powershell
   $env:DATABASE_URL="postgresql://rustyassets:rustyassets@127.0.0.1:5432/rustyassets"
   ```

   > **Performance Tip**: Using `127.0.0.1` instead of `localhost` can significantly improve connection speed on Windows (96.5% faster), avoiding DNS resolution delays.

3. Run migrations:

   ```powershell
   # Install SQLx CLI first
   cargo install sqlx-cli --no-default-features --features rustls,postgres

   # Run migrations
   sqlx migrate run --source crates/assets-core/migrations
   ```

### Testing and Coverage

Run tests with coverage analysis to track code quality:

```bash
# Run all tests
cargo test

# Generate HTML coverage report
cargo cov

# Generate XML coverage report for CI
cargo cov-ci
```

Coverage reports help identify untested code areas and track testing progress toward the 80% coverage goal.

### Project Status

✅ **Completed:**

- Double-entry bookkeeping system design
- Account types: Assets, Liabilities, Equity, Income, Expenses
- Transaction and journal entry models
- Database schema with PostgreSQL migrations
- Core business logic services
- CLI demo with interactive examples
- Rust workspace structure with proper separation
- Database connection and migration runner
- CRUD operations for accounts and transactions
- Investment price tracking and portfolio valuation
- Balance calculation from journal entries
- Price history tracking for investments
- Multi-user ownership with percentage allocation
- Deep account hierarchies with parent-child relationships
- Interactive account creation workflow

🔄 **Next Steps:**

- Real-world transaction import (CSV, QIF, OFX)
- Automated price feeds from financial APIs
- Web or GUI interface
- Reporting: balance sheets, income statements, net worth tracking
- GitHub Actions for CI/CD

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

# 📈 Investment Tracking
cargo run -- prices history             # View all tracked symbols with latest prices
cargo run -- prices history AAPL        # Detailed price history with % changes for AAPL
cargo run -- prices market              # Investment accounts with market values and gains/losses
cargo run -- demo create-sample-prices  # Create 30-day price history for 9 symbols

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

# 🆕 Account Management
cargo run -- accounts create            # Interactive account creation with 10-step workflow
```

### 🎯 Quick Test Sequence

```powershell
# 1. Create deep hierarchical data
cargo run -- demo create-deep-accounts

# 2. View the beautiful tree structure
cargo run -- accounts tree

# 3. See real balances calculated from transactions
cargo run -- accounts balance

# 4. Add price history for investment tracking
cargo run -- demo create-sample-prices

# 5. View investment portfolio performance
cargo run -- prices market

# 6. Learn the accounting principles
cargo run -- demo double-entry

# 7. Try different user perspectives
cargo run --user you -- accounts balance
```

### Manual PostgreSQL Setup

If you prefer to install PostgreSQL manually:

```powershell
# Install PostgreSQL and create database
createdb rustyassets

# Update .env with your connection details
# DATABASE_URL=postgresql://username:password@127.0.0.1:5432/rustyassets

# Note: Using 127.0.0.1 instead of localhost improves performance significantly on Windows

# Run migrations
cargo run --bin assets-cli -- init-db
```

### Database Commands

````powershell
### Database Commands

```powershell
# Initialize database and run migrations
cargo run -- db init

# Check connection and migration status
cargo run -- db status

# Create sample users and accounts
cargo run -- demo create-sample

# Create sample price history data
cargo run -- demo create-sample-prices

# Create deep account hierarchy
cargo run -- demo create-deep-accounts

# View multi-user examples
cargo run -- demo multi-user

# View fractional ownership examples
cargo run -- demo ownership
````

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
# Create sample data with accounts and transactions
cargo run -- demo create-sample

# See interactive examples
cargo run -- demo

# View account hierarchy as a tree
cargo run -- accounts tree

# Check account balances
cargo run -- accounts balance

# Investment tracking
cargo run -- prices market
cargo run -- prices history AAPL

# Learn about account types
cargo run -- demo account-types

# Multi-user and ownership examples
cargo run -- demo multi-user
cargo run -- demo ownership

# Try user context switching
cargo run --user you -- accounts balance
cargo run --user spouse -- accounts balance
cargo run --user family -- accounts balance

# Create a new account interactively
cargo run -- accounts create

# See all commands
cargo run -- --help
```

## Shell Completion

Generate shell completion scripts for faster command entry:

```bash
# Bash
assets-cli completion bash > ~/.bash_completion.d/assets-cli

# Zsh  
assets-cli completion zsh > ~/.zsh/completions/_assets-cli

# Fish
assets-cli completion fish > ~/.config/fish/completions/assets-cli.fish

# PowerShell
assets-cli completion powershell | Out-String | Invoke-Expression
```

## Contributing

Contributions and suggestions are welcome! See the project structure and add new crates for additional interfaces or features.

---

**Project name inspiration:**

- "RustyAssets" combines the app's goal (asset tracking) and the Rust language.
- Crate names are prefixed with `assets-` to avoid conflicts and clarify their purpose.
