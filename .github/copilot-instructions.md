<!-- Use this file to provide workspace-specific custom instructions to Copilot. For more details, visit https://code.visualstudio.com/docs/copilot/copilot-customization#_use-a-githubcopilotinstructionsmd-file -->

# Project: RustyAssets

- Multi-crate Rust workspace for personal finance tracking.
- Crates are in the `crates/` directory and prefixed with `assets-`.
- Use idiomatic Rust, modular design, and best practices for workspace organization.
- Core logic and DB in `assets-core`, CLI in `assets-cli`.
- PostgreSQL is the target database.
- Future interfaces (GUI/web) should be easy to add as new crates.
- When creating new crates with `cargo new`, always use `--vcs none` to avoid initializing git in subdirectories.
