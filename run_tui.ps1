#!/usr/bin/env powershell

# Navigate to the project root
Set-Location -Path c:\Users\nicol\code\rusty-assets

# Build and run the TUI application
Write-Host "Building and running RustyAssets TUI..."
cargo run --bin assets-tui
