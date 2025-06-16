pub mod basic_household;

// Re-export public functions for easier access
pub use basic_household::create_basic_household_demo;

// Placeholder functions for other demo scenarios
use anyhow::Result;

pub async fn create_investment_demo() -> Result<()> {
    anyhow::bail!("Investment demo not yet implemented")
}

pub async fn create_joint_finances_demo() -> Result<()> {
    anyhow::bail!("Joint finances demo not yet implemented")
}

pub async fn create_complete_demo() -> Result<()> {
    anyhow::bail!("Complete demo not yet implemented")
}
