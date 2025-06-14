use anyhow::Result;
use assets_core::models::CashFlowRow;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;

pub fn print_cash_flow_table(
    data: &[CashFlowRow],
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<()> {
    println!();
    println!("┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓");
    println!("┃                             📊 CASH FLOW STATEMENT                             ┃");
    println!(
        "┃                        {} - {}                         ┃",
        start_date.format("%B %d, %Y"),
        end_date.format("%B %d, %Y")
    );
    println!("┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛");
    println!();

    if data.is_empty() {
        println!("No cash flow data found for the selected period.");
        return Ok(());
    }

    // Group data by activity type
    let mut grouped_data: HashMap<String, Vec<&CashFlowRow>> = HashMap::new();
    for row in data {
        grouped_data
            .entry(row.activity_type.clone())
            .or_default()
            .push(row);
    }

    let mut net_operating = Decimal::ZERO;
    let mut net_investing = Decimal::ZERO;
    let mut net_financing = Decimal::ZERO;

    // Display each activity type
    for activity_type in ["Operating", "Investing", "Financing"] {
        let icon = match activity_type {
            "Operating" => "💰",
            "Investing" => "💼",
            "Financing" => "💳",
            _ => "📊",
        };

        println!("{} {} ACTIVITIES", icon, activity_type.to_uppercase());
        println!(
            "────────────────────────────────────────────────────────────────────────────────"
        );

        if let Some(activities) = grouped_data.get(activity_type) {
            // Group by category within the activity type
            let mut category_totals: HashMap<String, (Vec<&CashFlowRow>, Decimal)> = HashMap::new();

            for row in activities {
                let (rows, total) = category_totals
                    .entry(row.category_name.clone())
                    .or_insert((Vec::new(), Decimal::ZERO));
                rows.push(row);
                *total += row.cash_flow;
            }

            let mut activity_total = Decimal::ZERO;

            // Sort categories for consistent display
            let mut sorted_categories: Vec<_> = category_totals.iter().collect();
            sorted_categories.sort_by_key(|(name, _)| name.as_str());

            for (category_name, (rows, category_total)) in sorted_categories {
                if category_name != "Uncategorized" || rows.len() > 1 {
                    println!("   {}:", category_name);
                }

                for row in rows {
                    let amount_str = if row.cash_flow >= Decimal::ZERO {
                        format!("€ {:.2}", row.cash_flow)
                    } else {
                        format!("€ ({:.2})", row.cash_flow.abs())
                    };

                    let display_name = if category_name == "Uncategorized" && rows.len() == 1 {
                        &row.account_path
                    } else {
                        &row.account_path
                    };

                    println!(
                        "     └─ {:42} {:>12}",
                        truncate_string(display_name, 42),
                        amount_str
                    );
                }

                activity_total += category_total;
            }

            println!("                                                  ──────────────");
            let total_str = if activity_total >= Decimal::ZERO {
                format!("€ {:.2}", activity_total)
            } else {
                format!("€ ({:.2})", activity_total.abs())
            };
            println!(
                "   Net {} Cash Flow             {:>12}",
                activity_type, total_str
            );

            // Update totals
            match activity_type {
                "Operating" => net_operating = activity_total,
                "Investing" => net_investing = activity_total,
                "Financing" => net_financing = activity_total,
                _ => {}
            }
        } else {
            println!(
                "   (No {} activities in this period)",
                activity_type.to_lowercase()
            );
            println!("                                                  ──────────────");
            println!(
                "   Net {} Cash Flow                     € 0.00",
                activity_type
            );
        }

        println!();
    }

    // Summary
    let net_change = net_operating + net_investing + net_financing;

    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!(
        "   NET CHANGE IN CASH                        {:>12}",
        if net_change >= Decimal::ZERO {
            format!("€ {:.2}", net_change)
        } else {
            format!("€ ({:.2})", net_change.abs())
        }
    );
    println!("═══════════════════════════════════════════════════════════════════════════════");

    println!();
    println!("📊 Summary:");
    println!(
        "   Operating Cash Flow: {:>12}",
        format_amount(net_operating)
    );
    println!(
        "   Investing Cash Flow: {:>12}",
        format_amount(net_investing)
    );
    println!(
        "   Financing Cash Flow: {:>12}",
        format_amount(net_financing)
    );
    println!("   Net Change in Cash:  {:>12}", format_amount(net_change));

    println!();
    println!("💡 Tip: Use --format=csv or --format=json for data export");

    Ok(())
}

pub fn print_cash_flow_json(
    data: &[CashFlowRow],
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<()> {
    // Group data by activity type for summary
    let mut operating_total = Decimal::ZERO;
    let mut investing_total = Decimal::ZERO;
    let mut financing_total = Decimal::ZERO;

    for row in data {
        match row.activity_type.as_str() {
            "Operating" => operating_total += row.cash_flow,
            "Investing" => investing_total += row.cash_flow,
            "Financing" => financing_total += row.cash_flow,
            _ => {}
        }
    }

    let net_change = operating_total + investing_total + financing_total;

    let report = serde_json::json!({
        "report_type": "cash_flow_statement",
        "period": {
            "start_date": start_date,
            "end_date": end_date
        },
        "activities": data,
        "summary": {
            "operating_cash_flow": operating_total,
            "investing_cash_flow": investing_total,
            "financing_cash_flow": financing_total,
            "net_change_in_cash": net_change
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

pub fn print_cash_flow_csv(data: &[CashFlowRow]) -> Result<()> {
    // Print CSV header
    println!("Activity Type,Category,Account Name,Account Path,Cash Flow");

    // Print data rows
    for row in data {
        println!(
            "{},{},{},{},{:.2}",
            escape_csv_field(&row.activity_type),
            escape_csv_field(&row.category_name),
            escape_csv_field(&row.account_name),
            escape_csv_field(&row.account_path),
            row.cash_flow
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

fn format_amount(amount: Decimal) -> String {
    if amount >= Decimal::ZERO {
        format!("€ {:.2}", amount)
    } else {
        format!("€ ({:.2})", amount.abs())
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
