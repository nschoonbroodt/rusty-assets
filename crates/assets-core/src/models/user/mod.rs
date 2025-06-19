//! User-related models and types
//!
//! This module contains all types related to user management:
//! - Core user structures (User)
//! - User authentication and session management (future)

pub mod core;

// Re-export all public types for easier importing
pub use core::*;
