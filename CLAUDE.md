# CLAUDE.md - RustyAssets Development Guide

- Challenge me when about questionable choices.
- Explain your choices and argument
- Work in new branches, not in main

## Project Overview

RustyAssets is a modular, extensible personal finance tracker written in Rust using double-entry bookkeeping principles. The project uses a PostgreSQL database and follows a workspace structure with multiple crates.

When opening issues, use tag to configure them.
## Project Structure

```
rusty-assets/
    crates/
        assets-core/     # Core business logic and database models
            migrations/          # Database schema migrations (in assets-core)
        assets-cli/      # Command-line interface
        assets-demo/     # Demo data and examples
    docs/               # Documentation
    tasks/              # Development tasks and todos
```

## Key Technical Details

Perform computation in postgres if possible (with view, functions) over performing them in the rust code

### Database
- **Engine**: PostgreSQL 15+
- **Connection**: Uses 127.0.0.1 instead of localhost for better Windows performance
- **Migrations**: Located in `crates/assets-core/migrations/`
- **ORM**: SQLx with runtime queries (no compile-time macros)

### Architecture Principles
- **Double-entry bookkeeping**: All transactions must balance (debits = credits)
- **Multi-user support**: Fractional ownership percentages
- **Hierarchical accounts**: Unlimited nesting depth

### Account Types
- **Assets**: Bank accounts, investments, real estate
- **Liabilities**: Credit cards, loans, mortgages
- **Equity**: Owner's equity, retained earnings
- **Income**: Salary, dividends, interest
- **Expenses**: Groceries, utilities, rent

### Currency
- **Default**: EUR (€) for all transactions and accounts
- **Precision**: Decimal handling for financial calculations

## Development Workflow

### Adding New Features
1. Create/update database migrations in `assets-core/migrations/`
2. Update models in `assets-core/src/models.rs`
3. Add business logic in `assets-core/src/services/`
4. Add CLI commands in `assets-cli/src/commands/`
5. Write tests with appropriate coverage
6. Run quality checks: `cargo fmt`, `cargo clippy`, `cargo test`

### Testing and Coverage

#### Test Types
- **Unit Tests**: `cargo test --lib` - Tests individual modules/functions
- **All Tests**: `cargo test` - Runs all unit tests (no integration tests currently)
- **Specific Tests**: `cargo test <pattern>` - Run tests matching pattern

#### Coverage Commands
```bash
# Generate HTML coverage report (clean, excludes test utilities)
cargo cov --lib --ignore-filename-regex "test_utils\.rs$"

# Open coverage report in browser
cargo cov-open --lib --ignore-filename-regex "test_utils\.rs$"

# Generate coverage for CI (Cobertura XML format)
cargo cov-ci --lib --ignore-filename-regex "test_utils\.rs$"

# Generate text coverage report (clean)
cargo cov-text --lib --ignore-filename-regex "test_utils\.rs$"

# Quick coverage check (may include test utilities)
cargo cov-text --lib
```

**Note**: Always use `--lib --ignore-filename-regex "test_utils\.rs$"` for accurate coverage that excludes test helper code.

Uses `cargo-llvm-cov` for fast, accurate coverage analysis instead of tarpaulin.

### Database Schema Changes
1. Create new migration: `sqlx migrate add --source crates/assets-core/ <description>`
2. Write both up and down migrations
3. Test migration with `sqlx migrate run --source crates/assets-core/`
4. Update models and services accordingly

## Environment Setup

### Required Environment Variables
```bash
DATABASE_URL=postgresql://rustyassets:rustyassets@127.0.0.1:5432/rustyassets
```

### Docker Compose Services
- **PostgreSQL**: Port 5432, credentials in `docker-compose.yml`
- **Data persistence**: Uses Docker volume `rustyassets_postgres_data`

## Code Conventions

### Rust Style
- **Edition**: 2021
- **Linting**: Strict clippy mode (all warnings denied)
- **Formatting**: Standard rustfmt
- **Error Handling**: Custom error types in `assets-core/src/error.rs`

### Database Queries
- **Runtime queries**: No SQLx compile-time macros
- **Transactions**: Use database transactions for data consistency
- **Indexing**: Performance-optimized with appropriate indexes

## Troubleshooting

### Common Issues
- **Database connection**: Check Docker container status and DATABASE_URL
- **Migration errors**: Ensure PostgreSQL is running and accessible
