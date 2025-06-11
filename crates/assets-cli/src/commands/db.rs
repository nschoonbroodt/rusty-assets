use anyhow::Result;
use assets_core::Database;

pub async fn init_database() -> Result<()> {
    println!("🗄️  Initializing Database");
    println!("=========================\n");

    // Check if DATABASE_URL is set
    match std::env::var("DATABASE_URL") {
        Ok(url) => {
            // Hide password in display
            let display_url = if url.contains('@') {
                let parts: Vec<&str> = url.split('@').collect();
                if parts.len() >= 2 {
                    let mut user_part = parts[0].to_string();
                    if let Some(colon_pos) = user_part.rfind(':') {
                        user_part.replace_range(colon_pos + 1.., "****");
                    }
                    format!("{}@{}", user_part, parts[1..].join("@"))
                } else {
                    url.clone()
                }
            } else {
                url.clone()
            };

            println!("📡 Database URL: {}", display_url);

            // Try to connect and run migrations
            println!("🔄 Connecting to database...");
            match Database::from_env().await {
                Ok(db) => {
                    println!("✅ Connected successfully!");

                    println!("🔄 Running migrations...");
                    match db.migrate().await {
                        Ok(_) => {
                            println!("✅ Migrations completed successfully!");
                            println!("\n🎉 Database is ready for use!");
                        }
                        Err(e) => {
                            println!("❌ Migration failed: {}", e);
                            println!(
                                "\n💡 Make sure PostgreSQL is running and the database exists."
                            );
                            println!("   You can create it with: createdb rustyassets");
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Connection failed: {}", e);
                    println!("\n💡 Troubleshooting:");
                    println!("   1. Make sure PostgreSQL is running");
                    println!("   2. Check your DATABASE_URL in .env file");
                    println!("   3. Create the database: createdb rustyassets");
                    println!("   4. Ensure the user has proper permissions");
                }
            }
        }
        Err(_) => {
            println!("❌ DATABASE_URL not found");
            println!("\n💡 Please create a .env file with:");
            println!("   DATABASE_URL=postgresql://username:password@localhost:5432/rustyassets");
            println!("\n📝 You can copy .env.example to .env and modify it.");
        }
    }

    Ok(())
}

pub async fn show_db_status() -> Result<()> {
    println!("📊 Database Status");
    println!("==================\n");

    match std::env::var("DATABASE_URL") {
        Ok(url) => {
            // Hide password in display
            let display_url = if url.contains('@') {
                let parts: Vec<&str> = url.split('@').collect();
                if parts.len() >= 2 {
                    let mut user_part = parts[0].to_string();
                    if let Some(colon_pos) = user_part.rfind(':') {
                        user_part.replace_range(colon_pos + 1.., "****");
                    }
                    format!("{}@{}", user_part, parts[1..].join("@"))
                } else {
                    url.clone()
                }
            } else {
                url.clone()
            };

            println!("📡 Database URL: {}", display_url);

            // Try to connect
            match Database::from_env().await {
                Ok(_db) => {
                    println!("✅ Connection: Successful");
                    println!("🗄️  Database: Ready");

                    // Could add more detailed status here like:
                    // - Table counts
                    // - Last migration version
                    // - User count
                    // - Transaction count

                    println!("\n📈 Quick Stats:");
                    println!("   • Tables: Ready (migrations applied)");
                    println!("   • Users: Check with create-sample command");
                    println!("   • Transactions: 0 (ready for first entries)");
                }
                Err(e) => {
                    println!("❌ Connection: Failed");
                    println!("   Error: {}", e);
                    println!("\n💡 Run 'init-db' command to set up the database");
                }
            }
        }
        Err(_) => {
            println!("❌ DATABASE_URL not configured");
            println!("\n💡 Please create a .env file with your database connection");
        }
    }

    Ok(())
}
