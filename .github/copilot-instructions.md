<!-- Use this file to provide workspace-specific custom instructions to Copilot. For more details, visit https://code.visualstudio.com/docs/copilot/copilot-customization#_use-a-githubcopilotinstructionsmd-file -->

# Project: RustyAssets

- Multi-crate Rust workspace for personal finance tracking.
- Crates are in the `crates/` directory and prefixed with `assets-`.
- Use idiomatic Rust, modular design, and best practices for workspace organization.
- Core logic and DB in `assets-core`, CLI in `assets-cli`.
- PostgreSQL is the target database.
- Always use the non-macro version of SQLx queries (e.g., `sqlx::query()` instead of `sqlx::query!()`).
- prefer to create view in the database instead of using complex queries, so that the view can be reused in multiple places without code duplication.
- prefer to ask the database to do some calculations instead of doing them in Rust (join on tables, aggregate functions, etc.) to avoid multiple round trips.
- Future interfaces (GUI/web) should be easy to add as new crates.
- When creating new crates with `cargo new`, always use `--vcs none` to avoid initializing git in subdirectories.
- tell me to update the todo file when something is done
- if you identify missing features, update the tasks/todo.md file with new tasks
- suggest to commit when tasks are done and it make sense to do so
- don't mention the tasks/todo.md and tasks/done.md file in the commits
- in rust try to avoid clone when possible, use references instead
- when the user needs to identify an account, he should use the syntax "Assets:Current Assets:Main Checking", not refer by id
- don't hesitate to ask for more details if the task is not clear
- don't hesitate to say that my ideas are bad if you think so, and suggest alternatives
- don't hesite to say that you don't know how to do something, and suggest alternatives
- when doing migration using sqlx cli, add the argument '--source crates/assets-core/migrations' to the command