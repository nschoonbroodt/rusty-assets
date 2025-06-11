# RustyAssets TUI

A component-based terminal user interface for the RustyAssets personal finance tracker.

## Features

- Component-based architecture for modular and maintainable code
- Tab navigation between different screens (Accounts, Transactions, Reports, Settings)
- Interactive lists with keyboard navigation
- Styled output with colors and formatting

## Components

- **Accounts**: View and manage financial accounts
- **Transactions**: Track income and expenses
- **Reports**: Generate financial reports and visualizations
- **Settings**: Configure application settings

## Usage

To run the TUI application:

```bash
# From the workspace root
cargo run --bin assets-tui

# Or use the provided script
./run_tui.ps1
```

## Key Bindings

- `q`: Quit the application
- `1-4`: Navigate between tabs
- `Up/Down`: Navigate through lists
- `Enter`: Select an item
- `Left/Right`: Navigate between report tabs
