use anyhow::Result;
use assets_core::{AccountService, AccountType, Database};
use std::str::FromStr;
use uuid::Uuid;

pub async fn list_accounts() -> Result<()> {
    println!("📋 Account List");
    println!("===============\n");

    // Connect to database
    let db = Database::from_env().await?;
    let account_service = AccountService::new(db.pool().clone());

    // Get all accounts
    let accounts = account_service.get_all_accounts().await?;

    if accounts.is_empty() {
        println!("No accounts found. Create some sample data with:");
        println!("cargo run -- demo create-sample\n");
        return Ok(());
    }

    // Group accounts by type
    println!("📊 Accounts by Type:\n");

    for account_type in [
        AccountType::Asset,
        AccountType::Liability,
        AccountType::Equity,
        AccountType::Income,
        AccountType::Expense,
    ] {
        let type_accounts: Vec<_> = accounts
            .iter()
            .filter(|a| a.account_type == account_type)
            .collect();

        if !type_accounts.is_empty() {
            println!("🏷️  {:?} Accounts:", account_type);
            for account in type_accounts {
                let balance_indicator = match account_type {
                    AccountType::Asset | AccountType::Expense => "💰",
                    AccountType::Liability | AccountType::Equity | AccountType::Income => "💳",
                };
                println!(
                    "   {} {} - {} ({})",
                    balance_indicator,
                    account.code,
                    account.name,
                    format!("{:?}", account.account_subtype)
                );
                if let Some(notes) = &account.notes {
                    println!("      📝 {}", notes);
                }
            }
            println!();
        }
    }

    println!("💡 Use 'cargo run -- accounts tree' to see hierarchical view");
    println!("💡 Use 'cargo run -- accounts balance --id <account_id>' for balances");

    Ok(())
}

pub async fn show_account_balance(account_id: Option<&str>) -> Result<()> {
    println!("💰 Account Balance");
    println!("==================\n");

    // Connect to database
    let db = Database::from_env().await?;
    let account_service = AccountService::new(db.pool().clone());

    if let Some(id_str) = account_id {
        // Try to find account by code first, then by UUID
        let account = if let Ok(account_uuid) = Uuid::from_str(id_str) {
            // It's a valid UUID
            account_service.get_account(account_uuid).await?
        } else {
            // Try as account code
            account_service.get_account_by_code(id_str).await?
        };

        match account {
            Some(account) => {
                println!("📊 Account: {} - {}", account.code, account.name);
                println!(
                    "   Type: {:?} ({:?})",
                    account.account_type, account.account_subtype
                );
                println!("   Currency: {}", account.currency);

                if let Some(symbol) = &account.symbol {
                    println!("   Symbol: {}", symbol);
                }

                if let Some(quantity) = account.quantity {
                    println!("   Quantity: {}", quantity);
                }
                if let Some(avg_cost) = account.average_cost {
                    println!("   Average Cost: €{}", avg_cost);
                }

                // TODO: Calculate actual balance from journal entries
                println!("\n💡 Balance calculation from journal entries coming soon!");
            }
            None => {
                println!("❌ Account not found: {}", id_str);
            }
        }
    } else {
        // Show summary of all account balances
        let accounts = account_service.get_all_accounts().await?;

        if accounts.is_empty() {
            println!("No accounts found. Create some sample data with:");
            println!("cargo run -- demo create-sample\n");
            return Ok(());
        }

        println!("📈 Account Balance Summary:\n");

        for account_type in [
            AccountType::Asset,
            AccountType::Liability,
            AccountType::Equity,
            AccountType::Income,
            AccountType::Expense,
        ] {
            let type_accounts: Vec<_> = accounts
                .iter()
                .filter(|a| a.account_type == account_type)
                .collect();

            if !type_accounts.is_empty() {
                println!("📊 {:?} Accounts:", account_type);
                for account in type_accounts {
                    println!(
                        "   {} - {} (ID: {})",
                        account.code, account.name, account.id
                    );
                    // TODO: Show actual calculated balance
                    println!("      Balance: [Calculation coming soon]");
                }
                println!();
            }
        }

        println!("💡 Use --id <account_id> to see specific account details");
    }

    Ok(())
}

pub async fn create_account_interactive() -> Result<()> {
    println!("🏗️  Create New Account");
    println!("=====================\n");

    println!("This interactive account creation is coming soon!");
    println!("For now, you can create sample accounts with:");
    println!("cargo run -- demo create-sample\n");

    println!("🎯 Planned Features:");
    println!("   • Interactive prompts for account details");
    println!("   • Account type and subtype selection");
    println!("   • Automatic code generation");
    println!("   • Parent account selection for hierarchies");
    println!("   • Validation and confirmation");

    Ok(())
}

pub async fn show_accounts_tree() -> Result<()> {
    println!("🌳 Chart of Accounts Tree");
    println!("=========================\n");

    // Connect to database
    let db = Database::from_env().await?;
    let account_service = AccountService::new(db.pool().clone());

    let accounts = account_service.get_all_accounts().await?;

    if accounts.is_empty() {
        println!("No accounts found. Create some sample data with:");
        println!("cargo run -- demo create-sample\n");
        return Ok(());
    }

    // Build hierarchical tree structure
    println!("📊 Chart of Accounts (Hierarchical View):\n");

    // Group by account type and show hierarchy
    for account_type in [
        AccountType::Asset,
        AccountType::Liability,
        AccountType::Equity,
        AccountType::Income,
        AccountType::Expense,
    ] {
        let type_accounts: Vec<_> = accounts
            .iter()
            .filter(|a| a.account_type == account_type)
            .collect();

        if !type_accounts.is_empty() {
            let type_icon = match account_type {
                AccountType::Asset => "🏦",
                AccountType::Liability => "💳",
                AccountType::Equity => "🏠",
                AccountType::Income => "💰",
                AccountType::Expense => "💸",
            };

            println!("{} {:?} Accounts", type_icon, account_type);

            // Find root accounts (no parent) in this type
            let root_accounts: Vec<_> = type_accounts
                .iter()
                .filter(|a| a.parent_id.is_none())
                .collect();

            for (i, account) in root_accounts.iter().enumerate() {
                let is_last = i == root_accounts.len() - 1;
                print_account_tree_node(account, &type_accounts, "", is_last);
            }
            println!();
        }
    }

    println!("💡 Use 'cargo run -- accounts list' for a flat view");
    println!("💡 Parent-child relationships shown with tree structure");

    Ok(())
}

fn print_account_tree_node(
    account: &assets_core::Account,
    all_accounts: &[&assets_core::Account],
    prefix: &str,
    is_last: bool,
) {
    // Print current account
    let connector = if is_last { "└── " } else { "├── " };
    println!(
        "{}{}💼 {} - {}",
        prefix, connector, account.code, account.name
    );

    // Find children
    let children: Vec<_> = all_accounts
        .iter()
        .filter(|a| a.parent_id == Some(account.id))
        .collect();

    // Print children
    let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
    for (i, child) in children.iter().enumerate() {
        let is_last_child = i == children.len() - 1;
        print_account_tree_node(child, all_accounts, &new_prefix, is_last_child);
    }
}

pub async fn show_account_ownership(account_id: &str) -> Result<()> {
    println!("🏠 Account Ownership Details");
    println!("============================\n");

    // Connect to database
    let db = Database::from_env().await?;
    let account_service = AccountService::new(db.pool().clone());    // Try to find account by code first, then by UUID
    let account_with_ownership = if let Ok(account_uuid) = Uuid::from_str(account_id) {
        // It's a valid UUID
        account_service
            .get_account_with_ownership_and_users(account_uuid)
            .await?
    } else {
        // Try as account code - first get the account, then get ownership
        if let Some(account) = account_service.get_account_by_code(account_id).await? {
            account_service
                .get_account_with_ownership_and_users(account.id)
                .await?
        } else {
            None
        }
    };

    match account_with_ownership {
        Some(account_with_ownership) => {
            let account = &account_with_ownership.account;

            println!("📊 Account: {} - {}", account.code, account.name);
            println!(
                "   Type: {:?} ({:?})",
                account.account_type, account.account_subtype
            );
            println!("   Currency: {}", account.currency);
            println!();
            if account_with_ownership.ownership.is_empty() {
                println!("🏦 Ownership: 100% Unassigned (no specific owners)");
                println!("   This account has no fractional ownership setup.");
            } else {                println!("👥 Ownership Distribution:");
                for ownership in &account_with_ownership.ownership {
                    let percentage = ownership
                        .ownership_percentage
                        .to_string()
                        .parse::<f64>()
                        .unwrap_or(0.0);
                    println!(
                        "   • {}: {:.1}%",
                        ownership.user_display_name,
                        percentage * 100.0
                    );
                }

                let total_percentage: f64 = account_with_ownership
                    .ownership
                    .iter()
                    .map(|o| {
                        o.ownership_percentage
                            .to_string()
                            .parse::<f64>()
                            .unwrap_or(0.0)
                    })
                    .sum();

                println!();
                println!("📊 Total Ownership: {:.1}%", total_percentage * 100.0);

                if (total_percentage - 1.0).abs() > 0.001 {
                    println!("⚠️  Warning: Ownership does not sum to 100%!");
                }
            }
        }
        None => {
            println!("❌ Account not found: {}", account_id);
            println!("💡 Use 'cargo run -- accounts list' to see available accounts");
        }
    }

    Ok(())
}
