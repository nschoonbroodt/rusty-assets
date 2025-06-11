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

## Contributing

Contributions and suggestions are welcome! See the project structure and add new crates for additional interfaces or features.

---

**Project name inspiration:**

- "RustyAssets" combines the app's goal (asset tracking) and the Rust language.
- Crate names are prefixed with `assets-` to avoid conflicts and clarify their purpose.
