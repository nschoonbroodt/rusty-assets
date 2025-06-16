use anyhow::Result;
use assets_core::models::IncomeStatementRow;
use comfy_table::{presets::UTF8_FULL, ContentArrangement, Table};
use csv;
use rust_decimal::Decimal;
use serde_json;

/// Format and print income statement in a professional table format
pub(super) fn print_income_statement_table(data: &[IncomeStatementRow]) -> Result<()> {
    println!();
    println!("┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓");
    println!("┃                             📊 INCOME STATEMENT                                ┃");
    println!("┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛");
    println!();

    if data.is_empty() {
        // Added check for empty data
        println!("No income statement data to display for the selected criteria.");
        return Ok(());
    }
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Category", "Account Path", "Total Amount"]);

    for row in data {
        table.add_row(vec![
            row.category_name.as_deref().unwrap_or("N/A"),
            &row.account_path,
            &format_currency(row.total_amount),
        ]);
    }

    println!("{table}");

    // Footer notes (similar to balance sheet)
    // if params.include_zero { // Assuming IncomeStatementParams might have include_zero
    //     println!("📝 Note: Zero balances might be included if applicable.");
    // }
    println!("💡 Tip: Use --format=csv or --format=json for data export");

    Ok(())
}

/// Print income statement in JSON format
pub(super) fn print_income_statement_json(data: &[IncomeStatementRow]) -> Result<()> {
    if data.is_empty() {
        println!("[]"); // Print empty JSON array if no data
        return Ok(());
    }
    let json_output = serde_json::to_string_pretty(data)?;
    println!("{}", json_output);
    Ok(())
}

/// Print income statement in CSV format
pub(super) fn print_income_statement_csv(data: &[IncomeStatementRow]) -> Result<()> {
    if data.is_empty() {
        // Print header only if no data
        println!("Category,Account Name,Total Amount");
        return Ok(());
    }
    let mut wtr = csv::Writer::from_writer(std::io::stdout());
    wtr.write_record(["Category", "Account Name", "Total Amount"])?;
    for row in data {
        wtr.serialize(row)?;
    }
    wtr.flush()?;
    Ok(())
}

// Helper function, could be moved to a shared utility module
fn format_currency(amount: Decimal) -> String {
    if amount.is_sign_negative() {
        format!("€({:.2})", amount.abs())
    } else {
        format!("€ {:.2}", amount) // Added a space for positive numbers for alignment
    }
}
