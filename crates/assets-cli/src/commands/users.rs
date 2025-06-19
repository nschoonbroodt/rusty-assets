use anyhow::Result;
use assets_core::models::NewUser;
use assets_core::{Database, UserService};
use clap::{Args, Subcommand};
use comfy_table::{presets::UTF8_FULL, Table};

#[derive(Subcommand)]
pub enum UserCommands {
    /// List all users
    List,
    /// Add a new user
    Add(AddUserArgs),
    /// Get user by name (shows UUID)
    Get {
        /// User name to look up
        name: String,
    },
}

#[derive(Args)]
pub struct AddUserArgs {
    /// User name (unique identifier)
    #[arg(short, long)]
    name: String,

    /// Display name (can contain spaces and special characters)
    #[arg(short, long)]
    display_name: String,
}

pub async fn handle_user_command(command: UserCommands) -> Result<()> {
    match command {
        UserCommands::List => list_users().await,
        UserCommands::Add(args) => add_user(args).await,
        UserCommands::Get { name } => get_user_by_name(&name).await,
    }
}

async fn list_users() -> Result<()> {
    println!("👥 Users");
    println!("========\n");

    let db = Database::from_env().await?;
    let user_service = UserService::new(db.pool().clone());

    let users = user_service.get_all_users().await?;

    if users.is_empty() {
        println!("No users found. Create one with 'assets-cli users add --name <name> --display-name <display>'");
        return Ok(());
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["UUID", "Name", "Display Name", "Created"]);

    for user in users {
        table.add_row(vec![
            user.id.to_string(),
            user.name,
            user.display_name,
            user.created_at.format("%Y-%m-%d %H:%M").to_string(),
        ]);
    }

    println!("{table}");
    Ok(())
}

async fn add_user(args: AddUserArgs) -> Result<()> {
    println!("👤 Creating User");
    println!("================\n");

    let db = Database::from_env().await?;
    let user_service = UserService::new(db.pool().clone());

    // Check if user already exists
    if let Some(_existing) = user_service.get_user_by_name(&args.name).await? {
        return Err(anyhow::anyhow!(
            "User with name '{}' already exists. Use 'assets-cli users get {}' to see details.",
            args.name,
            args.name
        ));
    }

    let user = user_service
        .create_user(
            NewUser::builder()
                .name(args.name)
                .display_name(args.display_name)
                .build(),
        )
        .await?;

    println!("✅ User created successfully!");
    println!();
    println!("📋 User Details:");
    println!("   UUID: {}", user.id);
    println!("   Name: {}", user.name);
    println!("   Display Name: {}", user.display_name);
    println!("   Created: {}", user.created_at.format("%Y-%m-%d %H:%M"));
    println!();
    println!("💡 Save this UUID - you'll need it for other commands:");
    println!("   export USER_ID=\"{}\"", user.id);

    Ok(())
}

async fn get_user_by_name(name: &str) -> Result<()> {
    println!("🔍 Looking up User: {}", name);
    println!("====================={}\n", "=".repeat(name.len()));

    let db = Database::from_env().await?;
    let user_service = UserService::new(db.pool().clone());

    match user_service.get_user_by_name(name).await? {
        Some(user) => {
            println!("📋 User Details:");
            println!("   UUID: {}", user.id);
            println!("   Name: {}", user.name);
            println!("   Display Name: {}", user.display_name);
            println!("   Created: {}", user.created_at.format("%Y-%m-%d %H:%M"));
            println!();
            println!("💡 Copy this UUID for use in other commands:");
            println!("   {}", user.id);
        }
        None => {
            println!("❌ User '{}' not found.", name);
            println!();
            println!("💡 Create this user with:");
            println!(
                "   assets-cli users add --name {} --display-name \"Display Name\"",
                name
            );
        }
    }

    Ok(())
}
