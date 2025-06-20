# Installation Guide for RustyAssets Tauri Interface

## Prerequisites

### System Dependencies (Linux)

Install required system packages for Tauri development:

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
    librsvg2-dev \
    libglib2.0-dev \
    libgdk-pixbuf2.0-dev \
    libcairo2-dev \
    libpango1.0-dev \
    libatk1.0-dev
```

### Tauri CLI

The Tauri CLI is already installed in the project.

```bash
cargo tauri --version
```

## Development Setup

1. **Install Node.js dependencies**:
   ```bash
   cd crates/assets-tauri/ui
   npm install
   ```

2. **Start PostgreSQL database**:
   ```bash
   docker-compose up -d
   ```

3. **Initialize database** (if not already done):
   ```bash
   cargo run -p assets-cli -- db init
   ```

4. **Run the development server**:
   ```bash
   cd crates/assets-tauri
   cargo tauri dev
   ```

   This will:
   - Start the SvelteKit dev server on http://localhost:1420
   - Launch the Tauri desktop application
   - Enable hot-reload for both frontend and backend changes

## Building for Production

```bash
cd crates/assets-tauri
cargo tauri build
```

This creates:
- AppImage (Linux)
- .deb package (Debian/Ubuntu)
- Other platform-specific bundles

## Troubleshooting

### WSL/Linux Issues

If you're on WSL and get display errors:
- Install an X server like VcXsrv on Windows
- Set `DISPLAY` environment variable
- Or use GitHub Codespaces/remote development

### Missing System Dependencies

If you get pkg-config errors about missing libraries:
```bash
# Install the missing development packages
sudo apt install libglib2.0-dev libgtk-3-dev
```

### Permission Errors

If you get permission errors during build:
```bash
# Make sure your user is in the right groups
sudo usermod -a -G audio,video $USER
```

## Demo Data

To see the interface with sample data:

```bash
cargo run -p assets-cli -F demo -- demo create-sample
```

This will populate the database with example accounts and transactions.

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   SvelteKit     │    │   Tauri Core     │    │  assets-core    │
│   Frontend      │◄──►│   (Rust)         │◄──►│   Services      │
│                 │    │                  │    │                 │
│ - Dashboard     │    │ - Commands       │    │ - AccountService│
│ - Components    │    │ - Window Mgmt    │    │ - TransactionSvc│
│ - Charts        │    │ - File System    │    │ - ReportService │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

The interface connects your existing Rust business logic with a modern desktop UI.