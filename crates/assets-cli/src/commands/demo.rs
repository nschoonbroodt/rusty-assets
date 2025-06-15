use anyhow::Result;
use assets_demo::*;

use crate::DemoCommands;

pub async fn handle_demo_command(action: DemoCommands) -> Result<()> {
    match action {
        DemoCommands::BasicHousehold => create_basic_household_demo().await, //create_basic_household_demo().await,
        DemoCommands::Investment => create_investment_demo().await,
        DemoCommands::JointFinances => create_joint_finances_demo().await,
        DemoCommands::Complete => create_complete_demo().await,
    }
}
