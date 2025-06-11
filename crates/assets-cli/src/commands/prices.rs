use anyhow::Result;
use assets_core::{Database, PriceHistoryService, AccountService, NewPriceHistory};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::io::{self, Write};
use std::str::FromStr;

pub async fn add_price_interactive() -> Result<()> {
    println!("📈 Add Asset Price");
    println!("==================\n");

    let db = Database::from_env().await?;
    let price_service = PriceHistoryService::new(db.pool().clone());

    // Get symbol
    let symbol = prompt_input("Symbol (e.g., AAPL, SPY, BTC): ")?.to_uppercase();
    
    // Get price
    let price_input = prompt_input("Price in EUR: ")?;
    let price = Decimal::from_str(&price_input)
        .map_err(|_| anyhow::anyhow!("Invalid price format"))?;

    // Get date (default to today)
    let date_input = prompt_input("Date (YYYY-MM-DD, or Enter for today): ")?;
    let price_date = if date_input.is_empty() {
        chrono::Utc::now().naive_utc().date()
    } else {
        NaiveDate::from_str(&date_input)
            .map_err(|_| anyhow::anyhow!("Invalid date format. Use YYYY-MM-DD"))?
    };

    // Get source (optional)
    let source = prompt_input("Source (optional, e.g., 'manual', 'yahoo_finance'): ")?;
    let source = if source.is_empty() { 
        Some("manual".to_string()) 
    } else { 
        Some(source) 
    };

    println!("\n📋 Price Entry Summary:");
    println!("======================");
    println!("Symbol: {}", symbol);
    println!("Price: €{}", price);
    println!("Date: {}", price_date);
    if let Some(ref src) = source {
        println!("Source: {}", src);
    }

    if !confirm_entry()? {
        println!("❌ Price entry cancelled.");
        return Ok(());
    }

    let new_price = NewPriceHistory {
        symbol: symbol.clone(),
        price,
        price_date,
        currency: "EUR".to_string(),
        source,
    };

    match price_service.add_price(new_price).await {
        Ok(price_entry) => {
            println!("✅ Price added successfully!");
            println!("   ID: {}", price_entry.id);
            println!("   Symbol: {}", price_entry.symbol);
            println!("   Price: €{}", price_entry.price);
            println!("   Date: {}", price_entry.price_date);
        }
        Err(e) => {
            println!("❌ Failed to add price: {}", e);
        }
    }

    Ok(())
}

pub async fn show_price_history(symbol: Option<&str>) -> Result<()> {
    println!("📊 Price History");
    println!("================\n");

    let db = Database::from_env().await?;
    let price_service = PriceHistoryService::new(db.pool().clone());

    if let Some(symbol) = symbol {
        // Show history for specific symbol
        let prices = price_service.get_price_history(symbol, None, None).await?;
        
        if prices.is_empty() {
            println!("❌ No price history found for symbol: {}", symbol);
            println!("💡 Add prices with: cargo run -- prices add");
            return Ok(());
        }

        println!("📈 Price History for {}:", symbol.to_uppercase());
        println!("Date       | Price     | Source");
        println!("-----------|-----------|----------");

        for price in &prices {
            println!(
                "{} | €{:>8} | {}",
                price.price_date,
                format!("{:.2}", price.price),
                price.source.as_deref().unwrap_or("unknown")
            );
        }

        // Show price change
        if prices.len() > 1 {
            let first_price = &prices[0];
            let last_price = &prices[prices.len() - 1];
            let change = last_price.price - first_price.price;
            let change_percent = (change / first_price.price) * Decimal::from(100);
            
            println!("\n📊 Price Change:");
            println!("   From: €{} ({})", first_price.price, first_price.price_date);
            println!("   To:   €{} ({})", last_price.price, last_price.price_date);
            println!("   Change: €{} ({:.2}%)", change, change_percent);
        }

        // Show latest price prominently
        if let Some(latest) = prices.last() {
            println!("\n💰 Latest Price: €{} ({})", latest.price, latest.price_date);
        }

    } else {
        // Show all tracked symbols with latest prices
        let symbols = price_service.get_tracked_symbols().await?;
        
        if symbols.is_empty() {
            println!("❌ No price history found.");
            println!("💡 Add prices with: cargo run -- prices add");
            return Ok(());
        }

        println!("📈 All Tracked Symbols:");
        println!("Symbol | Latest Price | Latest Date");
        println!("-------|--------------|------------");

        for symbol in symbols {
            if let Ok(Some(latest_price)) = price_service.get_latest_price(&symbol).await {
                println!(
                    "{:6} | €{:>10} | {}",
                    symbol,
                    format!("{:.2}", latest_price.price),
                    latest_price.price_date
                );
            }
        }

        println!("\n💡 Use 'cargo run -- prices history <SYMBOL>' for detailed history");
    }

    Ok(())
}

pub async fn show_market_values() -> Result<()> {
    println!("📊 Investment Market Values");
    println!("===========================\n");

    let db = Database::from_env().await?;
    let account_service = AccountService::new(db.pool().clone());
    let price_service = PriceHistoryService::new(db.pool().clone());

    // Get all investment accounts (those with symbols)
    let all_accounts = account_service.get_all_accounts().await?;
    let investment_accounts: Vec<_> = all_accounts
        .into_iter()
        .filter(|a| a.symbol.is_some() && a.quantity.is_some())
        .collect();

    if investment_accounts.is_empty() {
        println!("❌ No investment accounts found.");
        println!("💡 Create investment accounts with: cargo run -- accounts create");
        return Ok(());
    }

    println!("📈 Investment Accounts with Market Values:\n");
    println!("Code | Name                | Symbol | Quantity | Book Value | Market Value | Gain/Loss");
    println!("-----|---------------------|--------|----------|------------|--------------|----------");

    let mut total_book_value = Decimal::ZERO;
    let mut total_market_value = Decimal::ZERO;

    for account in investment_accounts {
        let account_with_market = price_service.get_account_with_market_value(account).await?;
        
        total_book_value += account_with_market.book_value;
        
        let (market_val_str, gain_loss_str) = if let Some(market_value) = account_with_market.market_value {
            total_market_value += market_value;
            let gain_loss = account_with_market.unrealized_gain_loss.unwrap_or(Decimal::ZERO);
            let gain_loss_sign = if gain_loss >= Decimal::ZERO { "+" } else { "" };
            (
                format!("€{:.2}", market_value),
                format!("{}€{:.2}", gain_loss_sign, gain_loss)
            )
        } else {
            ("No price".to_string(), "N/A".to_string())
        };

        println!(
            "{:4} | {:19} | {:6} | {:8} | €{:9.2} | {:12} | {}",
            account_with_market.account.code,
            if account_with_market.account.name.len() > 19 {
                format!("{}...", &account_with_market.account.name[..16])
            } else {
                account_with_market.account.name.clone()
            },
            account_with_market.account.symbol.as_deref().unwrap_or(""),
            account_with_market.account.quantity.map(|q| format!("{:.2}", q)).unwrap_or_default(),
            account_with_market.book_value,
            market_val_str,
            gain_loss_str
        );
    }

    println!("-----|---------------------|--------|----------|------------|--------------|----------");
    let total_gain_loss = total_market_value - total_book_value;
    let total_gain_loss_sign = if total_gain_loss >= Decimal::ZERO { "+" } else { "" };
    
    println!(
        "     | {:19} | {:6} | {:8} | €{:9.2} | €{:10.2} | {}€{:.2}",
        "TOTAL",
        "",
        "",
        total_book_value,
        total_market_value,
        total_gain_loss_sign,
        total_gain_loss
    );

    if total_book_value > Decimal::ZERO {
        let total_return_percent = (total_gain_loss / total_book_value) * Decimal::from(100);
        println!("\n📊 Total Portfolio Performance:");
        println!("   Total Return: {}€{:.2} ({:.2}%)", total_gain_loss_sign, total_gain_loss, total_return_percent);
    }

    println!("\n💡 Use 'cargo run -- prices add' to update asset prices");
    println!("💡 Use 'cargo run -- prices history <SYMBOL>' for price trends");

    Ok(())
}

// Helper functions
fn prompt_input(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn confirm_entry() -> Result<bool> {
    loop {
        let input = prompt_input("\n✅ Add this price entry? (y/n): ")?;
        match input.to_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => println!("Please enter 'y' for yes or 'n' for no."),
        }
    }
}
