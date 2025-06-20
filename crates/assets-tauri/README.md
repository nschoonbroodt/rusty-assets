# RustyAssets Tauri Desktop Interface

This crate provides a desktop GUI interface for RustyAssets using [Tauri](https://tauri.app/) with a [SvelteKit](https://kit.svelte.dev/) frontend.

## Architecture

- **Backend**: Rust with Tauri, exposing `assets-core` services via commands
- **Frontend**: SvelteKit with TypeScript for the UI
- **Charts**: Chart.js for financial visualizations
- **Styling**: Modern glass-morphism design with gradients

## Features

- 💰 **Dashboard Overview**: Financial summary cards showing assets, liabilities, and net worth
- 📊 **Interactive Charts**: Doughnut charts showing account distribution
- 🏦 **Account Management**: Beautiful cards displaying all account details
- 📱 **Responsive Design**: Modern UI that works on different screen sizes

## Setup

### Prerequisites

For Linux development, install the required system dependencies:

```bash
sudo apt update
sudo apt install -y \
    libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

### Development

1. **Install Node.js dependencies**:
   ```bash
   cd crates/assets-tauri/ui
   npm install
   ```

2. **Start development server**:
   ```bash
   cd crates/assets-tauri
   cargo tauri dev
   ```

   This will:
   - Start the SvelteKit dev server
   - Launch the Tauri desktop application
   - Enable hot-reload for both frontend and backend changes

### Building

```bash
cd crates/assets-tauri
cargo tauri build
```

## Project Structure

```
crates/assets-tauri/
├── src/
│   ├── lib.rs          # Tauri backend with command handlers
│   └── main.rs         # Application entry point
├── ui/                 # SvelteKit frontend
│   ├── src/
│   │   ├── routes/
│   │   │   └── +page.svelte    # Main dashboard page
│   │   └── lib/
│   │       └── components/     # Reusable UI components
│   │           ├── Dashboard.svelte
│   │           ├── AccountCard.svelte
│   │           └── BalanceChart.svelte
│   └── package.json
├── Cargo.toml
└── tauri.conf.json     # Tauri configuration
```

## Tauri Commands

The following commands are exposed from the Rust backend to the frontend:

- `get_accounts()` - Fetch all accounts
- `get_account_by_id(id)` - Get specific account
- `get_transactions()` - Fetch all transactions  
- `get_balance_sheet()` - Generate balance sheet report
- `get_income_statement()` - Generate income statement report

## Frontend Components

### Dashboard
The main dashboard component that orchestrates the entire UI:
- Financial summary cards
- Balance overview chart
- Account listings organized by type

### AccountCard
Individual account display cards with:
- Account type icons
- Detailed account information
- Status indicators (active/inactive)
- Investment-specific fields (symbol, quantity)

### BalanceChart
Interactive doughnut chart showing:
- Account distribution by type
- Hover tooltips with percentages
- Color-coded visualization

## Development Notes

- Uses `@tauri-apps/api` for backend communication
- SvelteKit configured with static adapter for Tauri compatibility
- Chart.js integrated for financial visualizations
- Modern CSS with backdrop-filter for glass-morphism effects

## Limitations

- Currently requires system GTK dependencies on Linux
- Some features are placeholder (balance calculations need real backend data)
- Icons directory not yet created (will cause build warnings)

## Future Enhancements

- Add transaction creation/editing forms
- Implement real-time balance calculations
- Add more chart types (line charts for trends)
- Create account creation workflow
- Add data export functionality
- Implement dark/light theme toggle