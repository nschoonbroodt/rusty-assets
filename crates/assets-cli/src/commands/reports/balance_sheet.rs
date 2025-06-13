use anyhow::Result;
use assets_core::{AccountBalance, BalanceSheetData};

use super::BalanceSheetParams; // Assuming BalanceSheetParams is in the parent module

/// Format and print balance sheet in a professional table format
pub(super) fn print_balance_sheet_table(
    data: &BalanceSheetData,
    params: &BalanceSheetParams,
) -> Result<()> {
    // Header
    println!();
    println!("┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓");
    println!("┃                              📊 BALANCE SHEET                               ┃");
    println!(
        "┃                               As of {}                              ┃",
        data.report_date.format("%B %d, %Y")
    );
    println!("┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛");
    println!();

    // Assets Section
    println!("📈 ASSETS");
    println!("{}", "─".repeat(80));
    if data.assets.is_empty() {
        println!("   (No asset accounts with balances)");
    } else {
        print_account_section(&data.assets, "€");
    }
    println!("{}", "─".repeat(80));
    println!(
        "{:>60} {:>15}",
        "Total Assets:",
        format_currency(data.total_assets)
    );
    println!();

    // Liabilities Section
    println!("📉 LIABILITIES");
    println!("{}", "─".repeat(80));
    if data.liabilities.is_empty() {
        println!("   (No liability accounts with balances)");
    } else {
        print_account_section(&data.liabilities, "€");
    }
    println!("{}", "─".repeat(80));
    println!(
        "{:>60} {:>15}",
        "Total Liabilities:",
        format_currency(data.total_liabilities)
    );
    println!();

    // Equity Section
    println!("⚖️  EQUITY");
    println!("{}", "─".repeat(80));
    if data.equity.is_empty() {
        println!("   (No equity accounts with balances)");
    } else {
        print_account_section(&data.equity, "€");
    }
    println!("{}", "─".repeat(80));
    println!(
        "{:>60} {:>15}",
        "Total Equity:",
        format_currency(data.total_equity)
    );
    println!();

    // Net Worth (Assets - Liabilities)
    let net_worth = data.total_assets - data.total_liabilities;
    println!("{:>60} {:>15}", "Net Worth:", format_currency(net_worth));
    println!();

    // Footer notes
    println!();
    if params.include_zero {
        println!("📝 Note: Zero balances are included in this report.");
    } else {
        println!("📝 Note: Zero balances are excluded from this report.");
    }
    println!("💡 Tip: Use --include-zero to show accounts with zero balances");
    println!("💡 Tip: Use --format=csv or --format=json for data export");

    Ok(())
}

/// Print a section of accounts with proper indentation
fn print_account_section(accounts: &[AccountBalance], currency: &str) {
    for account in accounts {
        let indent = "  ".repeat(account.level as usize);
        let name_width = 60 - (account.level as usize * 2);

        // Show hierarchy with visual indicators
        let hierarchy_indicator = if account.level > 0 { "└─ " } else { "" };

        println!(
            "   {}{}{:<width$} {:>15}",
            indent,
            hierarchy_indicator,
            account.name,
            format!("{}{:.2}", currency, account.balance),
            width = name_width
        );
    }
}

/// Print balance sheet in JSON format
pub(super) fn print_balance_sheet_json(data: &BalanceSheetData) -> Result<()> {
    use serde_json::json;

    let output = json!({
        "report_type": "balance_sheet",
        "report_date": data.report_date,
        "currency": "EUR",
        "assets": data.assets.iter().map(|a| json!({
            "name": a.name,
            "full_path": a.full_path,
            "balance": a.balance,
            "level": a.level
        })).collect::<Vec<_>>(),
        "liabilities": data.liabilities.iter().map(|l| json!({
            "name": l.name,
            "full_path": l.full_path,
            "balance": l.balance,
            "level": l.level
        })).collect::<Vec<_>>(),
        "equity": data.equity.iter().map(|e| json!({
            "name": e.name,
            "full_path": e.full_path,
            "balance": e.balance,
            "level": e.level
        })).collect::<Vec<_>>(),
        "totals": {
            "total_assets": data.total_assets,
            "total_liabilities": data.total_liabilities,
            "total_equity": data.total_equity
        }
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Print balance sheet in CSV format
pub(super) fn print_balance_sheet_csv(data: &BalanceSheetData) -> Result<()> {
    println!("Report Type,Account Type,Account Name,Full Path,Balance,Level,Report Date");

    for asset in &data.assets {
        println!(
            "balance_sheet,asset,\"{}\",\"{}\",{},{},{}",
            asset.name, asset.full_path, asset.balance, asset.level, data.report_date
        );
    }

    for liability in &data.liabilities {
        println!(
            "balance_sheet,liability,\"{}\",\"{}\",{},{},{}",
            liability.name,
            liability.full_path,
            liability.balance,
            liability.level,
            data.report_date
        );
    }

    for equity in &data.equity {
        println!(
            "balance_sheet,equity,\"{}\",\"{}\",{},{},{}",
            equity.name, equity.full_path, equity.balance, equity.level, data.report_date
        );
    }

    // Summary rows
    println!(
        "balance_sheet,summary,\"Total Assets\",\"\",{},0,{}",
        data.total_assets, data.report_date
    );
    println!(
        "balance_sheet,summary,\"Total Liabilities\",\"\",{},0,{}",
        data.total_liabilities, data.report_date
    );
    println!(
        "balance_sheet,summary,\"Total Equity\",\"\",{},0,{}",
        data.total_equity, data.report_date
    );

    Ok(())
}

/// Format currency amounts consistently
fn format_currency(amount: rust_decimal::Decimal) -> String {
    if amount.is_sign_negative() {
        format!("€({:.2})", amount.abs())
    } else {
        format!("€{:.2}", amount)
    }
}
