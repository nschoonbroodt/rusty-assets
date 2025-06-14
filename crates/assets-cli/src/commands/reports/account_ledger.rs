use anyhow::Result;
use assets_core::models::{Account, AccountLedgerRow};
use chrono::NaiveDate;
use comfy_table::{Attribute, Cell, CellAlignment, Color, ContentArrangement, Table};
use rust_decimal::Decimal;

pub fn print_account_ledger_table(
    data: &[AccountLedgerRow],
    account: &Account,
    start_date: NaiveDate,
    end_date: NaiveDate,
    show_balance: bool,
) -> Result<()> {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);

    // Create header
    println!();
    println!("┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓");
    println!("┃                             📋 ACCOUNT LEDGER                                  ┃");
    println!(
        "┃                        {} - {}                            ┃",
        start_date.format("%B %d, %Y"),
        end_date.format("%B %d, %Y")
    );
    println!("┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛");
    println!();
    println!(
        "Account: {}",
        account.full_path.as_ref().unwrap_or(&account.name)
    );
    println!();

    if data.is_empty() {
        println!("No transactions found for the selected account and date range.");
        return Ok(());
    }

    // Set up table headers
    let mut headers = vec![
        "Date",
        "Description",
        "Reference",
        "Memo",
        "Debit",
        "Credit",
    ];
    if show_balance {
        headers.push("Balance");
    }

    table.set_header(
        headers
            .iter()
            .map(|h| Cell::new(h).add_attribute(Attribute::Bold).fg(Color::Blue))
            .collect::<Vec<_>>(),
    );

    // Add data rows
    for row in data {
        let mut cells = vec![
            Cell::new(row.transaction_date.format("%Y-%m-%d")),
            Cell::new(&row.description),
            Cell::new(&row.reference),
            Cell::new(&row.memo),
            Cell::new(if row.debit_amount > Decimal::ZERO {
                format!("€ {:.2}", row.debit_amount)
            } else {
                String::new()
            })
            .set_alignment(CellAlignment::Right),
            Cell::new(if row.credit_amount > Decimal::ZERO {
                format!("€ {:.2}", row.credit_amount)
            } else {
                String::new()
            })
            .set_alignment(CellAlignment::Right),
        ];

        if show_balance {
            cells.push(
                Cell::new(format!("€ {:.2}", row.running_balance))
                    .set_alignment(CellAlignment::Right)
                    .fg(if row.running_balance >= Decimal::ZERO {
                        Color::Green
                    } else {
                        Color::Red
                    }),
            );
        }

        table.add_row(cells);
    }

    println!("{}", table);

    // Show summary
    let total_debits: Decimal = data.iter().map(|r| r.debit_amount).sum();
    let total_credits: Decimal = data.iter().map(|r| r.credit_amount).sum();

    println!();
    println!("📊 Summary: {} transactions found", data.len());
    println!("   Total Debits:  € {:.2}", total_debits);
    println!("   Total Credits: € {:.2}", total_credits);

    if show_balance && !data.is_empty() {
        println!(
            "   Ending Balance: € {:.2}",
            data.last().unwrap().running_balance
        );
    }

    println!("💡 Tip: Use --format=csv or --format=json for data export");

    Ok(())
}

pub fn print_account_ledger_json(
    data: &[AccountLedgerRow],
    account: &Account,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<()> {
    let report = serde_json::json!({
        "report_type": "account_ledger",
        "account": {
            "id": account.id,
            "name": account.name,
            "full_path": account.full_path
        },
        "period": {
            "start_date": start_date,
            "end_date": end_date
        },
        "transactions": data,
        "summary": {
            "transaction_count": data.len(),
            "total_debits": data.iter().map(|r| r.debit_amount).sum::<Decimal>(),
            "total_credits": data.iter().map(|r| r.credit_amount).sum::<Decimal>(),
            "ending_balance": data.last().map(|r| r.running_balance)
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

pub fn print_account_ledger_csv(
    data: &[AccountLedgerRow],
    _account: &Account,
    _start_date: NaiveDate,
    _end_date: NaiveDate,
) -> Result<()> {
    // Print CSV header
    println!(
        "Date,Transaction ID,Description,Reference,Memo,Debit Amount,Credit Amount,Running Balance"
    );

    // Print data rows
    for row in data {
        println!(
            "{},{},{},{},{},{:.2},{:.2},{:.2}",
            row.transaction_date.format("%Y-%m-%d"),
            row.transaction_id,
            // Escape commas and quotes in text fields
            escape_csv_field(&row.description),
            escape_csv_field(&row.reference),
            escape_csv_field(&row.memo),
            row.debit_amount,
            row.credit_amount,
            row.running_balance
        );
    }

    Ok(())
}

fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}
