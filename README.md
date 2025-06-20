# RustyAssets

[![codecov](https://codecov.io/github/nschoonbroodt/rusty-assets/graph/badge.svg?token=HQ6I7ZDIT2)](https://codecov.io/github/nschoonbroodt/rusty-assets)

RustyAssets is a modular, extensible personal finance tracker written in Rust. It helps you track:

- **Bank accounts**
- **Spending by category**
- **Investments** (ETFs, shares, etc.)
- **Real estate** (property value and mortgage tracking)
- **Other assets** (easily extensible)

Note: This project is still in its early stage, use cautiously.

## Project Structure

This project is a Cargo workspace with all crates in the `crates/` directory:

- `assets-core`: Core logic, computation, and PostgreSQL database models/connection.
- `assets-cli`: Command-line interface for interacting with your data.
- `assets-demo`: Demo data injection. Activate with `demo` feature

## Features

- **Double-entry bookkeeping** with transaction validation
- **Hierarchical account structure** with unlimited nesting
- **Interactive account creation** with type-specific fields
- **Real-time balance calculation** from journal entries
- **PostgreSQL** for persistent, reliable storage
- **Modular design** for easy extension (e.g., add GUI/web interface as new crates)
- [partially implemented] **Reporting and analytics**

### Planned Features

- Recurring transactions
- Budgeting
- Net worth calculation
- Real-world transaction import (CSV, QIF, OFX)
- Automated price feeds from financial APIs
- Web or GUI interface
- Investment tracking with price history and market value calculations

## Getting Started with Docker (Recommended)

This is the recommended way to get RustyAssets up and running quickly.

1.  **Prerequisites**:

    - Install [Rust](https://www.rust-lang.org/tools/install).
    - Install [Docker Desktop](https://www.docker.com/products/docker-desktop/)
    
2.  **Clone the Repository**:

    ```bash
    git clone https://github.com/nschoonbroodt/rusty-assets/
    cd rustyassets
    ```

3.  **Configure Environment**:

    - Copy the example environment file:
      ```bash
      cp .env.example .env
      ```
    - Open the newly created `.env` file and verify the `DATABASE_URL`. It should match the credentials in `docker-compose.yml` (default: `postgresql://rustyassets:rustyassets@127.0.0.1:5432/rustyassets`).

4.  **Start PostgreSQL and initialize the database**:

    ```bash
    docker-compose up -d
    cargo run -- db init
    ```

    This command will:

    - Download the `postgres:15` image if you don't have it.
    - Start a PostgreSQL container with `rustyassets` db persisted in a volume
    - run the migration files (create the correct db tables and functions)


6.  **Build and run**:

    ```bash
    cargo build --workspace
    cargo run -p assets-cli -- --help
    ```

    This will show you the available commands.

8.  **(Optional) Create Sample Data**:
    ```
    cargo run -F demo -- demo create-sample
    ```


## Development

### Running Migrations

When you're ready to connect to a database:

1. Start PostgreSQL with Docker:

   ```bash
   docker run --name rustyassets-postgres -e POSTGRES_PASSWORD=rustyassets -e POSTGRES_USER=rustyassets -e POSTGRES_DB=rustyassets -p 5432:5432 -d postgres:16
   ```

2. Set your database URL:

   ```bash
   $env:DATABASE_URL="postgresql://rustyassets:rustyassets@127.0.0.1:5432/rustyassets"
   ```


3. Run migrations:

   ```bash
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

# Generate HTML coverage report and open it
cargo cov-open
```

## Testing Guide

After setting up the database and creating sample data, explore RustyAssets with these commands:

### 🧪 Essential Testing Commands

```powershell
# 📊 View Your Data
cargo run -- accounts tree              # Beautiful hierarchical chart of accounts
cargo run -- accounts balance           # All account balances from real transactions

# 📈 Investment Tracking
cargo run -- prices history             # View all tracked symbols with latest prices
cargo run -- prices history AAPL        # Detailed price history with % changes for AAPL
cargo run -- prices market              # Investment accounts with market values and gains/losses

# 🆕 Account Management
cargo run -- accounts create            # Interactive account creation with 10-step workflow
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
