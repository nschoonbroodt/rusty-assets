# Tauri MCP Plugin Integration Guide

## Overview
The Tauri MCP (Model Context Protocol) plugin enables AI agents to interact directly with Tauri applications for debugging and testing. This would allow Claude Code to:

- Capture screenshots of the actual Tauri window
- Simulate user interactions (clicks, typing)
- Execute JavaScript in the Tauri context
- Access DOM content and local storage
- Control window states and positioning

## Benefits for RustyAssets
- **Better debugging**: Direct interaction with the desktop app vs web interface
- **Automated testing**: AI-driven UI testing capabilities  
- **Enhanced development**: Real-time feedback and interaction during development

## Integration Steps

### 1. Clone the Plugin

#### Option A: Inside project (current setup)
```bash
cd /home/nicolas/code/rusty-assets
git clone https://github.com/P3GLEG/tauri-plugin-mcp.git
```

#### Option B: Outside project directory
```bash
cd /home/nicolas/code  # or any other location
git clone https://github.com/P3GLEG/tauri-plugin-mcp.git
```

#### Option C: Use Git dependency (no local clone needed)
Skip cloning entirely and use the Git dependency directly in Cargo.toml.

### 2. Add to Cargo.toml

Choose the appropriate dependency based on your clone location:

#### If cloned inside project:
```toml
[dependencies]
tauri-plugin-mcp = { path = "../../tauri-plugin-mcp" }
```

#### If cloned outside project:
```toml
[dependencies]
tauri-plugin-mcp = { path = "/home/nicolas/code/tauri-plugin-mcp" }
```

#### If using Git dependency (recommended):
```toml
[dependencies]
tauri-plugin-mcp = { git = "https://github.com/P3GLEG/tauri-plugin-mcp" }
```

### 3. Configure in Rust Code
Update `crates/assets-tauri/src/lib.rs`:
```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init());
    
    // Only add MCP plugin in development mode
    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_mcp::init_with_config(
            tauri_plugin_mcp::PluginConfig::new("RustyAssets")
        ));
    }
    
    builder
        .invoke_handler(tauri::generate_handler![
            get_accounts,
            get_account_by_id,
            get_transactions,
            get_balance_sheet,
            get_income_statement
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 4. Test Integration
After integration:
1. Restart `cargo tauri dev`
2. The MCP server should start automatically in debug mode
3. Check logs for MCP connection details
4. Test with MCP-compatible tools

## Connection Modes
- **IPC** (default): Platform-specific local sockets
- **TCP**: For Docker/remote debugging (if needed)

## Expected Capabilities
Once integrated, AI agents can:
- Take screenshots of the actual Tauri window (not just web interface)
- Interact with the real desktop application
- Test database connectivity through the actual Tauri backend
- Simulate real user workflows

## Claude Code Configuration

Once the Tauri MCP plugin is integrated and running, configure Claude Code to connect:

### Method 1: Auto-discovery (if supported)
Claude Code may automatically discover the MCP server when it starts with your Tauri app.

### Method 2: Manual Configuration
Add to your Claude Code configuration (typically `~/.claude/config.json` or project-specific config):

```json
{
  "mcpServers": {
    "tauri-rustyassets": {
      "command": "connect-to-existing",
      "transport": {
        "type": "ipc",
        "name": "RustyAssets"
      }
    }
  }
}
```

### Method 3: TCP Connection (if IPC doesn't work)
If you configure the plugin for TCP mode:

```json
{
  "mcpServers": {
    "tauri-rustyassets": {
      "transport": {
        "type": "tcp",
        "host": "localhost",
        "port": 3001
      }
    }
  }
}
```

### Verification Commands
After integration, you should be able to:
- `/mcp tauri-rustyassets screenshot` - Take a screenshot of the Tauri window
- `/mcp tauri-rustyassets click 100 200` - Click at coordinates
- `/mcp tauri-rustyassets eval "document.title"` - Execute JavaScript

## Next Steps
1. Complete the integration steps above  
2. Add the plugin dependency to Cargo.toml
3. Restart `cargo tauri dev`
4. Configure Claude Code MCP connection
5. Test screenshot and interaction capabilities