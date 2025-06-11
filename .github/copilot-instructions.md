<!-- Use this file to provide workspace-specific custom instructions to Copilot. For more details, visit https://code.visualstudio.com/docs/copilot/copilot-customization#_use-a-githubcopilotinstructionsmd-file -->

# Project: RustyAssets

- Multi-crate Rust workspace for personal finance tracking.
- Crates are in the `crates/` directory and prefixed with `assets-`.
- Use idiomatic Rust, modular design, and best practices for workspace organization.
- Core logic and DB in `assets-core`, CLI in `assets-cli`.
- PostgreSQL is the target database.
- Always use the non-macro version of SQLx queries (e.g., `sqlx::query()` instead of `sqlx::query!()`).
- prefer to ask the database to do some calculations instead of doing them in Rust (join on tables, aggregate functions, etc.) to avoid multiple round trips.
- Future interfaces (GUI/web) should be easy to add as new crates.
- When creating new crates with `cargo new`, always use `--vcs none` to avoid initializing git in subdirectories.
- move completed tasks to the bottom of the list when todo are done
- suggest to commit when tasks are done and it make sense to do so
- don't mention the TODO.md file in the commits
- try to avoid clone when possible, use references instead
