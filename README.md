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

- Database connection and migration runner
- CRUD operations for accounts and transactions
- Real-world transaction import (CSV, QIF, OFX)
- Investment price tracking and portfolio valuation
- Web or GUI interface
- Reporting: balance sheets, income statements, net worth tracking

## Architecture

RustyAssets uses **double-entry bookkeeping**, the same accounting system used by businesses and professional financial software. This provides:

- **Complete audit trail** - see exactly where every dollar comes from and goes
- **Built-in validation** - all transactions must balance (debits = credits)
- **Professional reporting** - can generate balance sheets, income statements, etc.
- **Unified account system** - cash, investments, real estate are all accounts

### Double-Entry Examples

**Getting paid $3,000 salary:**

```
Debit:  Checking Account (Asset)     +$3,000
Credit: Salary Income               +$3,000
```

**Buying $150 groceries with credit card:**

```
Debit:  Groceries Expense           +$150
Credit: Credit Card (Liability)     +$150
```

**Purchasing $2,500 of AAPL stock:**

```
Debit:  AAPL Stock (Asset)          +$2,500
Credit: Checking Account (Asset)    -$2,500
```

## Quick Demo

Try the built-in demo to see double-entry bookkeeping in action:

```powershell
# See interactive examples
cargo run -p assets-cli -- demo

# Learn about account types
cargo run -p assets-cli -- account-types

# See all commands
cargo run -p assets-cli -- --help
```

## Contributing

Contributions and suggestions are welcome! See the project structure and add new crates for additional interfaces or features.

---

**Project name inspiration:**

- "RustyAssets" combines the app's goal (asset tracking) and the Rust language.
- Crate names are prefixed with `assets-` to avoid conflicts and clarify their purpose.
