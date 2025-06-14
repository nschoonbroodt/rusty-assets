use anyhow::Result;
use assets_core::{AccountService, AccountSubtype, AccountType, Database, NewAccount, UserService};
use rust_decimal::Decimal;
use std::io::{self, Write};
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
                    "   {} {} ({})",
                    balance_indicator,
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

pub async fn show_account_balance(account_id_str: Option<&str>) -> Result<()> {
    println!("💰 Account Balance");
    println!("==================\n");

    // Connect to database
    let db = Database::from_env().await?;
    let account_service = AccountService::new(db.pool().clone());

    if let Some(id_str) = account_id_str {
        // Expect a UUID for specific account balance
        match Uuid::from_str(id_str) {
            Ok(account_uuid) => {
                match account_service.get_account(account_uuid).await? {
                    Some(account) => {
                        println!("📊 Account: {} (ID: {})", account.name, account.id);
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

                        // Calculate actual balance from journal entries
                        println!();
                        match account.calculate_balance(db.pool()).await {
                            Ok(balance) => {
                                // Format balance according to account type
                                let formatted_balance = if account.account_type
                                    == AccountType::Liability
                                    || account.account_type == AccountType::Equity
                                    || account.account_type == AccountType::Income
                                {
                                    // For credit accounts, show positive balance as the normal balance
                                    format!("€{:.2}", -balance)
                                } else {
                                    // For debit accounts (Assets, Expenses), show balance as-is
                                    format!("€{:.2}", balance)
                                };

                                let balance_type = if account.account_type.increases_with_debit() {
                                    "Debit balance"
                                } else {
                                    "Credit balance"
                                };

                                println!(
                                    "💰 Current Balance: {} ({})",
                                    formatted_balance, balance_type
                                );

                                if balance == rust_decimal::Decimal::ZERO {
                                    println!(
                                        "   Account has no activity or transactions cancel out"
                                    );
                                } else if balance > rust_decimal::Decimal::ZERO
                                    && account.account_type.increases_with_debit()
                                {
                                    println!(
                                        "   Positive balance - normal for {:?} accounts",
                                        account.account_type
                                    );
                                } else if balance < rust_decimal::Decimal::ZERO
                                    && account.account_type.increases_with_credit()
                                {
                                    println!(
                                        "   Normal balance for {:?} accounts",
                                        account.account_type
                                    );
                                }
                            }
                            Err(e) => {
                                println!("❌ Error calculating balance: {}", e);
                            }
                        }
                    }
                    None => {
                        println!("❌ Account not found with ID: {}", id_str);
                        println!("💡 Please provide a valid Account ID (UUID). Use 'accounts list' or 'accounts tree' to find IDs.");
                    }
                }
            }
            Err(_) => {
                println!(
                    "❌ Invalid Account ID format: {}. Please provide a valid UUID.",
                    id_str
                );
                println!("💡 Use 'accounts list' or 'accounts tree' to find account IDs.");
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
                    println!("   {} (ID: {})", account.name, account.id);

                    // Calculate actual balance from journal entries
                    match account.calculate_balance(db.pool()).await {
                        Ok(balance) => {
                            // Format balance according to account type
                            let formatted_balance = if account.account_type
                                == AccountType::Liability
                                || account.account_type == AccountType::Equity
                                || account.account_type == AccountType::Income
                            {
                                // For credit accounts, show positive balance as the normal balance
                                format!("€{:.2}", -balance)
                            } else {
                                // For debit accounts (Assets, Expenses), show balance as-is
                                format!("€{:.2}", balance)
                            };

                            if balance == rust_decimal::Decimal::ZERO {
                                println!("      Balance: {} (zero)", formatted_balance);
                            } else {
                                println!("      Balance: {}", formatted_balance);
                            }
                        }
                        Err(e) => {
                            println!("      Balance: [Error calculating: {}]", e);
                        }
                    }
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
    println!("=====================\n"); // Connect to database
    let db = Database::from_env().await?;
    let account_service = AccountService::new(db.pool().clone());
    let user_service = UserService::new(db.pool().clone());

    // Step 1: Select account type
    let account_type = prompt_account_type()?;

    // Step 2: Select account subtype
    let account_subtype = prompt_account_subtype(&account_type)?;

    // Step 3: Enter account name (Code is removed)
    let name = prompt_account_name()?;

    // Step 4: Select parent account (optional)
    let parent_id = prompt_parent_account(&account_service, &account_type).await?;

    // Step 5: Enter additional fields based on account type
    let (symbol, quantity, average_cost, address, purchase_date, purchase_price) =
        prompt_additional_fields(&account_type, &account_subtype)?;

    // Step 6: Enter notes (optional)
    let notes = prompt_notes()?;

    // Step 7: Show summary and confirm
    println!("\n📋 Account Summary:");
    println!("==================");
    // println!("Code: {}", code); // Code removed
    println!("Name: {}", name);
    println!("Type: {:?} ({:?})", account_type, account_subtype);
    if let Some(parent) = parent_id {
        if let Ok(Some(parent_account)) = account_service.get_account(parent).await {
            println!(
                "Parent: {} (ID: {})",
                parent_account.name, parent_account.id
            );
        }
    }
    if let Some(ref symbol) = symbol {
        println!("Symbol: {}", symbol);
    }
    if let Some(ref address) = address {
        println!("Address: {}", address);
    }
    if let Some(ref notes) = notes {
        println!("Notes: {}", notes);
    }

    if !confirm_creation()? {
        println!("❌ Account creation cancelled.");
        return Ok(());
    }

    // Step 8: Create the account
    let new_account = NewAccount {
        // code: code.clone(), // Code removed
        name: name.clone(),
        account_type: account_type.clone(),
        account_subtype,
        parent_id,
        symbol,
        quantity,
        average_cost,
        address,
        purchase_date,
        purchase_price,
        currency: "EUR".to_string(), // Default currency, consider making this a prompt
        notes,
    };
    println!("\n🔄 Creating account...");

    // Step 9: Set up ownership data before creating account (if requested)
    let ownership_data = if prompt_setup_ownership()? {
        prompt_account_ownership(&user_service).await?
    } else {
        Vec::new()
    };

    // Step 10: Create account with ownership in a single transaction
    match account_service
        .create_account_with_ownership(new_account, ownership_data)
        .await
    {
        Ok(account) => {
            println!("✅ Account created successfully!");
            println!("   ID: {}", account.id);
            // println!("   Code: {}", account.code); // Code removed
            println!("   Name: {}", account.name);

            println!("\n🎉 Account setup complete!");
            println!("\n💡 Next steps:");
            println!(
                "   • View with: cargo run -- accounts balance --id {}",
                account.id // Use ID instead of code
            );
            println!("   • See tree: cargo run -- accounts tree");
            println!("   • Create transactions involving this account");
        }
        Err(e) => {
            println!("❌ Failed to create account: {}", e);
            println!("💡 This could be due to:");
            println!("   • Account name conflict (name must be unique under the same parent, or globally if no parent)");
            println!("   • Invalid ownership percentages");
            println!("   • Database connection issues");
        }
    }

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
    println!("{}{}💼 {}", prefix, connector, account.name);

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

pub async fn show_account_ownership(account_id_str: &str) -> Result<()> {
    println!("🏠 Account Ownership Details");
    println!("============================\n");

    // Connect to database
    let db = Database::from_env().await?;
    let account_service = AccountService::new(db.pool().clone());

    // Expect a UUID for specific account ownership
    match Uuid::from_str(account_id_str) {
        Ok(account_uuid) => {
            match account_service
                .get_account_with_ownership_and_users(account_uuid)
                .await?
            {
                Some(account_with_ownership) => {
                    let account = &account_with_ownership.account;

                    println!("📊 Account: {} (ID: {})", account.name, account.id);
                    println!(
                        "   Type: {:?} ({:?})",
                        account.account_type, account.account_subtype
                    );
                    println!("   Currency: {}", account.currency);
                    println!();
                    if account_with_ownership.ownership.is_empty() {
                        println!("🏦 Ownership: 100% Unassigned (no specific owners)");
                        println!("   This account has no fractional ownership setup.");
                    } else {
                        println!("👥 Ownership Distribution:");
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
                    println!("❌ Account not found with ID: {}", account_id_str);
                    println!("💡 Please provide a valid Account ID (UUID). Use 'accounts list' or 'accounts tree' to find IDs.");
                }
            }
        }
        Err(_) => {
            println!(
                "❌ Invalid Account ID format: {}. Please provide a valid UUID.",
                account_id_str
            );
            println!("💡 Use 'accounts list' or 'accounts tree' to find account IDs.");
        }
    }

    Ok(())
}

/// Create account with command-line arguments or interactively
pub async fn create_account(
    name: Option<&str>,
    account_type: Option<&str>,
    subtype: Option<&str>,
    parent: Option<&str>,
    symbol: Option<&str>,
    currency: &str,
    notes: Option<&str>,
) -> Result<()> {
    // If any required argument is missing, fall back to interactive mode
    if name.is_none() || account_type.is_none() || subtype.is_none() {
        return create_account_interactive().await;
    }

    println!("🏗️  Create New Account (Command Line)");
    println!("=====================================\n");

    let db = Database::from_env().await?;
    let account_service = AccountService::new(db.pool().clone());

    // Parse account type
    let account_type = match account_type.unwrap().to_lowercase().as_str() {
        "asset" => AccountType::Asset,
        "liability" => AccountType::Liability,
        "equity" => AccountType::Equity,
        "income" => AccountType::Income,
        "expense" => AccountType::Expense,
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid account type. Must be one of: Asset, Liability, Equity, Income, Expense"
            ));
        }
    };

    // Parse account subtype
    let account_subtype = match subtype.unwrap().to_lowercase().as_str() {
        // Asset subtypes
        "cash" => AccountSubtype::Cash,
        "checking" => AccountSubtype::Checking,
        "savings" => AccountSubtype::Savings,
        "investmentaccount" => AccountSubtype::InvestmentAccount,
        "stocks" => AccountSubtype::Stocks,
        "etf" => AccountSubtype::Etf,
        "bonds" => AccountSubtype::Bonds,
        "mutualfund" => AccountSubtype::MutualFund,
        "crypto" => AccountSubtype::Crypto,
        "realestate" => AccountSubtype::RealEstate,
        "equipment" => AccountSubtype::Equipment,
        "otherasset" => AccountSubtype::OtherAsset,
        // Liability subtypes
        "creditcard" => AccountSubtype::CreditCard,
        "loan" => AccountSubtype::Loan,
        "mortgage" => AccountSubtype::Mortgage,
        "otherliability" => AccountSubtype::OtherLiability,
        // Equity subtypes
        "openingbalance" => AccountSubtype::OpeningBalance,
        "retainedearnings" => AccountSubtype::RetainedEarnings,
        "ownerequity" => AccountSubtype::OwnerEquity,
        // Income subtypes
        "salary" => AccountSubtype::Salary,
        "bonus" => AccountSubtype::Bonus,
        "dividend" => AccountSubtype::Dividend,
        "interest" => AccountSubtype::Interest,
        "investment" => AccountSubtype::Investment,
        "rental" => AccountSubtype::Rental,
        "otherincome" => AccountSubtype::OtherIncome,
        // Expense subtypes
        "otherexpense" => AccountSubtype::OtherExpense,
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid account subtype. See help for valid subtypes"
            ));
        }
    };

    // Parse parent account if provided
    let parent_id = if let Some(parent_path) = parent {
        match account_service.get_account_by_path(parent_path).await {
            Ok(parent_account) => Some(parent_account.id),
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "Parent account '{}' not found. Create it first or check the path",
                    parent_path
                ));
            }
        }
    } else {
        None
    };

    // Create the account
    let new_account = NewAccount {
        name: name.unwrap().to_string(),
        account_type: account_type.clone(),
        account_subtype,
        parent_id,
        symbol: symbol.map(|s| s.to_string()),
        quantity: None,
        average_cost: None,
        address: None,
        purchase_date: None,
        purchase_price: None,
        currency: currency.to_string(),
        notes: notes.map(|n| n.to_string()),
    };

    // Show summary
    println!("📋 Account Summary:");
    println!("==================");
    println!("Name: {}", new_account.name);
    println!(
        "Type: {:?} ({:?})",
        account_type, new_account.account_subtype
    );
    if let Some(parent_id) = parent_id {
        if let Ok(Some(parent_account)) = account_service.get_account(parent_id).await {
            println!(
                "Parent: {} ({})",
                parent_account.name,
                parent_account
                    .full_path
                    .unwrap_or_else(|| "No path".to_string())
            );
        }
    }
    if let Some(ref symbol) = new_account.symbol {
        println!("Symbol: {}", symbol);
    }
    if let Some(ref notes) = new_account.notes {
        println!("Notes: {}", notes);
    }
    println!("Currency: {}", new_account.currency);

    // Create the account
    match account_service.create_account(new_account).await {
        Ok(account) => {
            println!("\n✅ Account created successfully!");
            println!("📋 Account Details:");
            println!("   ID: {}", account.id);
            println!("   Name: {}", account.name);
            if let Some(full_path) = &account.full_path {
                println!("   Full Path: {}", full_path);
            }
            println!(
                "   Type: {:?} ({:?})",
                account.account_type, account.account_subtype
            );
            println!("   Currency: {}", account.currency);
            println!(
                "   Created: {}",
                account.created_at.format("%Y-%m-%d %H:%M")
            );
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to create account: {}", e));
        }
    }

    Ok(())
}

// Helper functions for interactive account creation

fn prompt_input(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn prompt_account_type() -> Result<AccountType> {
    println!("📊 Select Account Type:");
    println!("1. Asset (Cash, investments, property, etc.)");
    println!("2. Liability (Credit cards, loans, mortgages)");
    println!("3. Equity (Owner's equity, retained earnings)");
    println!("4. Income (Salary, dividends, capital gains)");
    println!("5. Expense (Food, utilities, taxes, etc.)");

    loop {
        let input = prompt_input("\nEnter choice (1-5): ")?;
        match input.as_str() {
            "1" => return Ok(AccountType::Asset),
            "2" => return Ok(AccountType::Liability),
            "3" => return Ok(AccountType::Equity),
            "4" => return Ok(AccountType::Income),
            "5" => return Ok(AccountType::Expense),
            _ => println!("❌ Invalid choice. Please enter 1-5."),
        }
    }
}

fn prompt_account_subtype(account_type: &AccountType) -> Result<AccountSubtype> {
    println!("\n📋 Select Account Subtype for {:?}:", account_type);

    let options = match account_type {
        AccountType::Asset => vec![
            (AccountSubtype::Cash, "Cash"),
            (AccountSubtype::Checking, "Checking Account"),
            (AccountSubtype::Savings, "Savings Account"),
            (AccountSubtype::InvestmentAccount, "Investment Account"),
            (AccountSubtype::Stocks, "Individual Stocks"),
            (AccountSubtype::Etf, "ETFs"),
            (AccountSubtype::Bonds, "Bonds"),
            (AccountSubtype::MutualFund, "Mutual Funds"),
            (AccountSubtype::Crypto, "Cryptocurrency"),
            (AccountSubtype::RealEstate, "Real Estate"),
            (AccountSubtype::Equipment, "Equipment"),
            (AccountSubtype::OtherAsset, "Other Asset"),
        ],
        AccountType::Liability => vec![
            (AccountSubtype::CreditCard, "Credit Card"),
            (AccountSubtype::Loan, "Personal Loan"),
            (AccountSubtype::Mortgage, "Mortgage"),
            (AccountSubtype::OtherLiability, "Other Liability"),
        ],
        AccountType::Equity => vec![
            (AccountSubtype::OpeningBalance, "Opening Balance Equity"),
            (AccountSubtype::RetainedEarnings, "Retained Earnings"),
            (AccountSubtype::OwnerEquity, "Owner's Equity"),
        ],
        AccountType::Income => vec![
            (AccountSubtype::Salary, "Salary"),
            (AccountSubtype::Bonus, "Bonus"),
            (AccountSubtype::Dividend, "Dividends"),
            (AccountSubtype::Interest, "Interest Income"),
            (AccountSubtype::Investment, "Investment Income"),
            (AccountSubtype::Rental, "Rental Income"),
            (AccountSubtype::CapitalGains, "Capital Gains"),
            (AccountSubtype::OtherIncome, "Other Income"),
        ],
        AccountType::Expense => vec![
            (AccountSubtype::Food, "Food & Dining"),
            (AccountSubtype::Housing, "Housing"),
            (AccountSubtype::Transportation, "Transportation"),
            (AccountSubtype::Utilities, "Utilities"),
            (AccountSubtype::Communication, "Phone & Internet"),
            (AccountSubtype::Entertainment, "Entertainment"),
            (AccountSubtype::Personal, "Personal Care"),
            (AccountSubtype::Healthcare, "Healthcare"),
            (AccountSubtype::Taxes, "Taxes"),
            (AccountSubtype::Fees, "Bank Fees"),
            (AccountSubtype::OtherExpense, "Other Expense"),
        ],
    };

    for (i, (_, name)) in options.iter().enumerate() {
        println!("{}. {}", i + 1, name);
    }

    loop {
        let input = prompt_input(&format!("\nEnter choice (1-{}): ", options.len()))?;
        if let Ok(choice) = input.parse::<usize>() {
            if choice >= 1 && choice <= options.len() {
                return Ok(options[choice - 1].0.clone());
            }
        }
        println!(
            "❌ Invalid choice. Please enter a number between 1 and {}.",
            options.len()
        );
    }
}

fn prompt_account_name() -> Result<String> {
    loop {
        let name = prompt_input("\n💼 Account Name: ")?;
        if !name.is_empty() && name.len() <= 255 {
            return Ok(name);
        }
        println!("❌ Account name is required and must be 255 characters or less.");
    }
}

pub async fn prompt_parent_account(
    _account_service: &AccountService, // Renamed to indicate it's not used directly in this function variant
    _account_type: &AccountType,       // Renamed to indicate it's not used
) -> Result<Option<Uuid>> {
    println!("\n🔗 Parent Account (Optional)");
    print!("Do you want to set a parent account? (y/N): ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() != "y" {
        return Ok(None);
    }

    // List potential parent accounts (same type or broader categories)
    // For simplicity, listing all accounts here. Could be refined.
    let all_accounts = _account_service.get_all_accounts().await?;
    if all_accounts.is_empty() {
        println!("No existing accounts to set as parent.");
        return Ok(None);
    }

    println!("Available accounts to set as parent:");
    for acc in &all_accounts {
        println!("  - {} (ID: {})", acc.name, acc.id);
    }

    loop {
        print!("Enter parent account ID (or leave blank for none): ");
        io::stdout().flush()?;
        input.clear();
        io::stdin().read_line(&mut input)?;
        let parent_id_str = input.trim();

        if parent_id_str.is_empty() {
            return Ok(None);
        }

        match Uuid::from_str(parent_id_str) {
            Ok(uuid) => {
                // Verify parent account exists
                if _account_service.get_account(uuid).await?.is_some() {
                    return Ok(Some(uuid));
                } else {
                    println!(
                        "❌ Parent account with ID {} not found. Please try again.",
                        uuid
                    );
                }
            }
            Err(_) => {
                println!("❌ Invalid UUID format. Please enter a valid parent account ID.");
            }
        }
    }
}

fn prompt_additional_fields(
    account_type: &AccountType,
    account_subtype: &AccountSubtype,
) -> Result<(
    Option<String>,
    Option<Decimal>,
    Option<Decimal>,
    Option<String>,
    Option<chrono::DateTime<chrono::Utc>>,
    Option<Decimal>,
)> {
    let mut symbol = None;
    let mut quantity = None;
    let mut average_cost = None;
    let mut address = None;
    let purchase_date = None;
    let mut purchase_price = None;

    // Asset-specific fields
    if *account_type == AccountType::Asset {
        match account_subtype {
            AccountSubtype::Stocks | AccountSubtype::Etf | AccountSubtype::Crypto => {
                println!("\n📈 Investment Details:");
                let input = prompt_input("Symbol (e.g., AAPL, BTC): ")?;
                if !input.is_empty() {
                    symbol = Some(input.to_uppercase());
                }

                let input = prompt_input("Quantity owned (optional): ")?;
                if !input.is_empty() {
                    if let Ok(qty) = Decimal::from_str(&input) {
                        quantity = Some(qty);
                    }
                }

                let input = prompt_input("Average cost per unit in EUR (optional): ")?;
                if !input.is_empty() {
                    if let Ok(cost) = Decimal::from_str(&input) {
                        average_cost = Some(cost);
                    }
                }
            }
            AccountSubtype::RealEstate => {
                println!("\n🏠 Real Estate Details:");
                let input = prompt_input("Address: ")?;
                if !input.is_empty() {
                    address = Some(input);
                }

                let input = prompt_input("Purchase price in EUR (optional): ")?;
                if !input.is_empty() {
                    if let Ok(price) = Decimal::from_str(&input) {
                        purchase_price = Some(price);
                    }
                }
            }
            _ => {}
        }
    }

    Ok((
        symbol,
        quantity,
        average_cost,
        address,
        purchase_date,
        purchase_price,
    ))
}

fn prompt_notes() -> Result<Option<String>> {
    let input = prompt_input("\n📝 Notes (optional): ")?;
    if input.is_empty() {
        Ok(None)
    } else {
        Ok(Some(input))
    }
}

fn confirm_creation() -> Result<bool> {
    loop {
        let input = prompt_input("\n✅ Create this account? (y/n): ")?;
        match input.to_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => println!("Please enter 'y' for yes or 'n' for no."),
        }
    }
}

fn prompt_setup_ownership() -> Result<bool> {
    loop {
        let input = prompt_input("\n👥 Set up account ownership (multi-user)? (y/n): ")?;
        match input.to_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => println!("Please enter 'y' for yes or 'n' for no."),
        }
    }
}

async fn prompt_account_ownership(user_service: &UserService) -> Result<Vec<(Uuid, Decimal)>> {
    println!("\n👥 Account Ownership Setup");
    println!("=========================");

    // Get all users
    let users = user_service.get_all_users().await?;

    if users.is_empty() {
        println!("❌ No users found. Create users first with sample data.");
        return Ok(Vec::new());
    }

    println!("Available users:");
    for (i, user) in users.iter().enumerate() {
        println!("{}. {} ({})", i + 1, user.display_name, user.name);
    }

    let mut ownership_data = Vec::new();
    let mut total_percentage = Decimal::from(0);

    loop {
        let input = prompt_input(&format!(
            "\nSelect user (1-{}, or Enter to finish): ",
            users.len()
        ))?;

        if input.is_empty() {
            break;
        }

        if let Ok(choice) = input.parse::<usize>() {
            if choice >= 1 && choice <= users.len() {
                let user = &users[choice - 1];
                let percentage_input = prompt_input(&format!(
                    "Ownership percentage for {} (0-100): ",
                    user.display_name
                ))?;
                if let Ok(percentage) = Decimal::from_str(&percentage_input) {
                    if percentage >= Decimal::from(0) && percentage <= Decimal::from(100) {
                        let decimal_percentage = percentage / Decimal::from(100); // Convert percentage to decimal
                        if total_percentage + decimal_percentage <= Decimal::from(1) {
                            ownership_data.push((user.id, decimal_percentage));
                            total_percentage += decimal_percentage;
                            println!(
                                "✅ Added {}% ownership for {}",
                                percentage, user.display_name
                            );

                            if total_percentage == Decimal::from(1) {
                                println!("💯 Total ownership is now 100%");
                                break;
                            }
                        } else {
                            println!(
                                "❌ Total ownership cannot exceed 100%. Current total: {}%",
                                total_percentage * Decimal::from(100)
                            );
                        }
                    } else {
                        println!("❌ Percentage must be between 0 and 100.");
                    }
                } else {
                    println!("❌ Invalid percentage format.");
                }
            } else {
                println!("❌ Invalid user choice.");
            }
        }
    }

    Ok(ownership_data)
}

/// Set opening balance for an account
pub async fn set_account_opening_balance(
    account_path: &str,
    amount: Decimal,
    date: Option<chrono::NaiveDate>,
    user: Option<&str>,
) -> Result<()> {
    use assets_core::{NewJournalEntry, NewTransaction, TransactionService};
    use chrono::{Datelike, Utc};

    let db = Database::from_env().await?;
    let account_service = AccountService::new(db.pool().clone());
    let user_service = UserService::new(db.pool().clone());
    let transaction_service = TransactionService::new(db.pool().clone());

    // Find the account
    println!("🔍 Looking up account: {}", account_path);
    let account = account_service.get_account_by_path(account_path).await
        .map_err(|_| anyhow::anyhow!("Account '{}' not found", account_path))?;

    // Find or use first user
    let target_user = if let Some(username) = user {
        match user_service.get_user_by_name(username).await? {
            Some(user) => user,
            None => return Err(anyhow::anyhow!("User '{}' not found", username)),
        }
    } else {
        // Get first user as default
        let users = user_service.get_all_users().await?;
        if users.is_empty() {
            return Err(anyhow::anyhow!("No users found. Create a user first."));
        }
        users.into_iter().next().unwrap()
    };

    // Find or create opening balance equity account
    let opening_balance_account = match account_service.get_account_by_path("Equity:Opening Balance").await {
        Ok(account) => account,
        Err(_) => {
            println!("📝 Creating 'Equity:Opening Balance' account...");
            
            // Find or create parent Equity account
            let equity_parent = match account_service.get_account_by_path("Equity").await {
                Ok(account) => Some(account.id),
                Err(_) => {
                    println!("📝 Creating 'Equity' parent account...");                    let equity_account = NewAccount {
                        name: "Equity".to_string(),
                        account_type: AccountType::Equity,
                        account_subtype: AccountSubtype::OwnerEquity,
                        parent_id: None,
                        symbol: None,
                        quantity: None,
                        average_cost: None,
                        address: None,
                        purchase_date: None,
                        purchase_price: None,
                        currency: "EUR".to_string(),
                        notes: Some("Parent equity account".to_string()),
                    };
                    Some(account_service.create_account(equity_account).await?.id)
                }
            };            let opening_balance_new = NewAccount {
                name: "Opening Balance".to_string(),
                account_type: AccountType::Equity,
                account_subtype: AccountSubtype::OpeningBalance,
                parent_id: equity_parent,
                symbol: None,
                quantity: None,
                average_cost: None,
                address: None,
                purchase_date: None,
                purchase_price: None,
                currency: "EUR".to_string(),
                notes: Some("Opening balances for accounts".to_string()),
            };
            account_service.create_account(opening_balance_new).await?
        }
    };

    // Use provided date or default to January 1st of current year
    let transaction_date = date.unwrap_or_else(|| {
        let current_year = Utc::now().year();
        chrono::NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap()
    });

    // Create the opening balance transaction
    // For assets: Debit the account, Credit opening balance
    // For liabilities: Credit the account, Debit opening balance
    let (account_amount, opening_balance_amount) = match account.account_type {
        AccountType::Asset => (amount, -amount), // Debit asset, Credit opening balance
        AccountType::Liability => (-amount, amount), // Credit liability, Debit opening balance
        _ => return Err(anyhow::anyhow!("Opening balances are only supported for Asset and Liability accounts")),
    };    let transaction = NewTransaction {
        description: format!("Opening balance for {}", account.name),
        reference: Some("OPENING".to_string()),
        transaction_date: transaction_date.and_hms_opt(12, 0, 0).unwrap().and_utc(),
        created_by: Some(target_user.id),
        entries: vec![
            NewJournalEntry {
                account_id: account.id,
                amount: account_amount,
                memo: Some("Opening balance".to_string()),
            },
            NewJournalEntry {
                account_id: opening_balance_account.id,
                amount: opening_balance_amount,
                memo: Some(format!("Opening balance for {}", account.name)),
            },
        ],
        import_source: None,
        import_batch_id: None,
        external_reference: None,
    };

    // Validate the transaction balances
    if !transaction.is_balanced() {
        return Err(anyhow::anyhow!("Opening balance transaction does not balance"));
    }

    // Create the transaction
    let created_transaction = transaction_service.create_transaction(transaction).await?;

    println!();
    println!("✅ Opening balance set successfully!");
    println!("   📊 Account: {}", account_path);
    println!("   💰 Amount: € {:.2}", amount);
    println!("   📅 Date: {}", transaction_date);
    println!("   👤 User: {}", target_user.name);
    println!("   🔗 Transaction ID: {}", created_transaction.transaction.id);
    println!();
    println!("💡 Tip: Run 'cargo run -- reports balance-sheet' to see the updated balance sheet");

    Ok(())
}
