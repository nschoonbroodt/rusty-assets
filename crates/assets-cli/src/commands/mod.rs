pub mod accounts;
pub mod db;
pub mod demo;
pub mod prices;

// Modules are exported but not their contents
// Main.rs imports them directly with `use commands::{accounts::*, db::*, demo::*, prices};`
