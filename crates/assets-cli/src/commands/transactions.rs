use anyhow::Result;
use assets_core::{Database, TransactionService, TransactionWithEntriesAndAccounts};
use chrono::{DateTime, NaiveDate, Utc};
use clap::{Args, Subcommand};
use comfy_table::{presets::UTF8_FULL, Table};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::OutputFormat;

#[derive(Subcommand)]
pub enum TransactionCommands {
    /// List transactions with optional filtering
    List(ListTransactionsArgs),
    /// Show detailed view of a specific transaction
    Show {
        /// Transaction ID to show
        id: String,
    },
    /// Detect and interactively merge potential internal transfers
    MergeTransfers {
        /// Date range start (YYYY-MM-DD format)
        #[arg(long)]
        from: Option<String>,
        /// Date range end (YYYY-MM-DD format)  
        #[arg(long)]
        to: Option<String>,
        /// Automatically confirm all merges without prompting
        #[arg(long)]
        auto_confirm: bool,
        /// Allow merging transactions with same date/amount but different descriptions
        #[arg(long)]
        allow_different_descriptions: bool,
    },
}

#[derive(Args)]
pub struct ListTransactionsArgs {
    /// Start date (YYYY-MM-DD format)
    #[arg(long)]
    from: Option<String>,

    /// End date (YYYY-MM-DD format)
    #[arg(long)]
    to: Option<String>,

    /// Filter by account path (e.g., "Assets:Current Assets:BoursoBank")
    #[arg(long)]
    account: Option<String>,

    /// Maximum number of transactions to show
    #[arg(long, default_value = "50")]
    limit: u32,

    /// Output format
    #[arg(long, value_enum, default_value = "table")]
    format: OutputFormat,
}

pub async fn handle_transaction_command(command: TransactionCommands) -> Result<()> {
    match command {
        TransactionCommands::List(args) => list_transactions(args).await,
        TransactionCommands::Show { id } => show_transaction(&id).await,
        TransactionCommands::MergeTransfers {
            from,
            to,
            auto_confirm,
            allow_different_descriptions,
        } => merge_internal_transfers(from, to, auto_confirm, allow_different_descriptions).await,
    }
}

async fn list_transactions(args: ListTransactionsArgs) -> Result<()> {
    println!("💰 Transactions");
    println!("===============\n");

    let db = Database::from_env().await?;
    let transaction_service = TransactionService::new(db.pool().clone());

    // Parse date filters
    let from_date = if let Some(from_str) = &args.from {
        Some(parse_date(from_str)?)
    } else {
        None
    };

    let to_date = if let Some(to_str) = &args.to {
        Some(parse_date(to_str)?)
    } else {
        None
    };
    // Get transactions with filters
    let transactions = transaction_service
        .get_transactions_with_filters_and_accounts(
            from_date,
            to_date,
            args.account.as_deref(),
            args.limit,
        )
        .await?;

    if transactions.is_empty() {
        println!("No transactions found with the specified filters.");
        println!();
        println!("💡 Try adjusting your filters or check:");
        println!("   - Date range with --from and --to");
        println!("   - Account filter with --account");
        println!("   - User filter with --user");
        return Ok(());
    }

    match args.format {
        OutputFormat::Table => display_transactions_table(&transactions),
        OutputFormat::Json => display_transactions_json(&transactions)?,
        OutputFormat::Csv => display_transactions_csv(&transactions)?,
    }

    println!();
    println!("📊 Summary: {} transactions found", transactions.len());
    if let (Some(from), Some(to)) = (&args.from, &args.to) {
        println!("📅 Date range: {} to {}", from, to);
    }

    Ok(())
}

async fn show_transaction(id_str: &str) -> Result<()> {
    let transaction_id = Uuid::parse_str(id_str)?;

    println!("🔍 Transaction Details");
    println!("======================\n");

    let db = Database::from_env().await?;
    let transaction_service = TransactionService::new(db.pool().clone());
    match transaction_service
        .get_transaction_with_accounts(transaction_id)
        .await?
    {
        Some(transaction_with_entries) => {
            display_transaction_detail(&transaction_with_entries);
        }
        None => {
            println!("❌ Transaction not found: {}", id_str);
            println!();
            println!("💡 Use 'cargo run -- transactions list' to see available transactions");
        }
    }

    Ok(())
}

fn parse_date(date_str: &str) -> Result<DateTime<Utc>> {
    let naive_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|_| anyhow::anyhow!("Invalid date format. Use YYYY-MM-DD (e.g., 2025-06-13)"))?;

    Ok(naive_date.and_hms_opt(0, 0, 0).unwrap().and_utc())
}

fn display_transactions_table(transactions: &[TransactionWithEntriesAndAccounts]) {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        "Date",
        "Description",
        "Entries",
        "Amount",
        "Reference",
        "ID",
    ]);

    for tx in transactions {
        let total_amount: Decimal =
            tx.entries.iter().map(|e| e.amount.abs()).sum::<Decimal>() / Decimal::from(2); // Divide by 2 since double-entry
        let entry_count = tx.entries.len();
        let reference = tx.transaction.reference.as_deref().unwrap_or("-");

        table.add_row(vec![
            tx.transaction
                .transaction_date
                .format("%Y-%m-%d")
                .to_string(),
            truncate_string(&tx.transaction.description, 30),
            format!("{} entries", entry_count),
            format!("{:.2}", total_amount),
            truncate_string(reference, 15),
            tx.transaction.id.to_string()[..8].to_string(), // Show first 8 chars of UUID
        ]);
    }

    println!("{table}");
}

fn display_transactions_json(transactions: &[TransactionWithEntriesAndAccounts]) -> Result<()> {
    let json = serde_json::to_string_pretty(transactions)?;
    println!("{}", json);
    Ok(())
}

fn display_transactions_csv(transactions: &[TransactionWithEntriesAndAccounts]) -> Result<()> {
    println!("Date,Description,Entries,Amount,Reference,ID");

    for tx in transactions {
        let total_amount: Decimal =
            tx.entries.iter().map(|e| e.amount.abs()).sum::<Decimal>() / Decimal::from(2);
        let entry_count = tx.entries.len();
        let reference = tx.transaction.reference.as_deref().unwrap_or("");

        println!(
            "{},{},{},{:.2},{},{}",
            tx.transaction.transaction_date.format("%Y-%m-%d"),
            escape_csv(&tx.transaction.description),
            entry_count,
            total_amount,
            escape_csv(reference),
            tx.transaction.id
        );
    }

    Ok(())
}

fn display_transaction_detail(transaction_with_entries: &TransactionWithEntriesAndAccounts) {
    let tx = &transaction_with_entries.transaction;

    println!("📋 Transaction Information:");
    println!("   ID: {}", tx.id);
    println!("   Description: {}", tx.description);
    println!("   Date: {}", tx.transaction_date.format("%Y-%m-%d %H:%M"));
    if let Some(ref reference) = tx.reference {
        println!("   Reference: {}", reference);
    }
    println!("   Created at: {}", tx.created_at.format("%Y-%m-%d %H:%M"));

    println!();
    println!("📊 Journal Entries:");

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        "Account Path",
        "Account Name",
        "Amount",
        "Type",
        "Memo",
    ]);

    for entry in &transaction_with_entries.entries {
        let (amount_str, entry_type) = if entry.amount >= Decimal::ZERO {
            (format!("+{:.2}", entry.amount), "Debit")
        } else {
            (format!("{:.2}", entry.amount), "Credit")
        };

        let memo = entry.memo.as_deref().unwrap_or("-");

        table.add_row(vec![
            truncate_string(&entry.account_path, 35),
            truncate_string(&entry.account_name, 20),
            amount_str,
            entry_type.to_string(),
            truncate_string(memo, 20),
        ]);
    }

    println!("{table}");

    // Verify balance
    let total: Decimal = transaction_with_entries
        .entries
        .iter()
        .map(|e| e.amount)
        .sum();
    println!();
    if total == Decimal::ZERO {
        println!("✅ Transaction is balanced (total: {:.2})", total);
    } else {
        println!("⚠️ Transaction is unbalanced (total: {:.2})", total);
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

async fn merge_internal_transfers(
    from: Option<String>,
    to: Option<String>,
    auto_confirm: bool,
    allow_different_descriptions: bool,
) -> Result<()> {
    println!("🔄 Internal Transfer Detection");
    println!("=============================\n");

    let db = Database::from_env().await?;
    let transaction_service = TransactionService::new(db.pool().clone());

    // Parse date filters
    let from_date = if let Some(from_str) = &from {
        Some(parse_date(from_str)?)
    } else {
        None
    };

    let to_date = if let Some(to_str) = &to {
        Some(parse_date(to_str)?)
    } else {
        None
    };

    // Find potential internal transfers using SQL
    let potential_transfers = find_potential_internal_transfers(
        &db,
        from_date.map(|d| d.date_naive()),
        to_date.map(|d| d.date_naive()),
        allow_different_descriptions,
    )
    .await?;

    if potential_transfers.is_empty() {
        println!("✅ No potential internal transfers found.");
        return Ok(());
    }

    println!(
        "🔍 Found {} potential internal transfer groups:",
        potential_transfers.len()
    );
    println!();

    for (i, group) in potential_transfers.iter().enumerate() {
        display_transfer_group(i + 1, group);

        if !auto_confirm {
            println!("Do you want to merge this transfer group? (y/N): ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() != "y" {
                println!("❌ Skipping this group.\n");
                continue;
            }
        }

        // Perform the merge
        match merge_transfer_group(&transaction_service, group).await {
            Ok(new_transaction_id) => {
                println!(
                    "✅ Successfully merged transfer group into transaction: {}",
                    new_transaction_id
                );
            }
            Err(e) => {
                println!("❌ Failed to merge transfer group: {}", e);
            }
        }
        println!();
    }

    Ok(())
}

#[derive(Debug)]
struct PotentialTransfer {
    transaction_id: Uuid,
    description: String,
    amount: Decimal,
    date: DateTime<Utc>,
    non_equity_account: String,
    non_equity_account_id: Uuid,
}

#[derive(Debug)]
struct TransferGroup {
    date: DateTime<Utc>,
    amount: Decimal,
    transfers: Vec<PotentialTransfer>,
}

async fn find_potential_internal_transfers(
    db: &Database,
    from_date: Option<NaiveDate>,
    to_date: Option<NaiveDate>,
    allow_different_descriptions: bool,
) -> Result<Vec<TransferGroup>> {
    // For now, let's create a simpler implementation without direct sqlx access
    // We'll get transactions with entries and filter in Rust

    let transaction_service = TransactionService::new(db.pool().clone());

    // Get transactions in date range
    let from_datetime = from_date.map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc());
    let to_datetime = to_date.map(|d| d.and_hms_opt(23, 59, 59).unwrap().and_utc());
    let transactions = transaction_service
        .get_transactions_with_filters_and_accounts(from_datetime, to_datetime, None, 1000)
        .await?;

    let mut potential_transfers = Vec::new(); // Find transactions that involve Equity:Uncategorized
    for tx in transactions {
        let has_equity = tx
            .entries
            .iter()
            .any(|e| e.account_path == "Equity:Uncategorized");
        if !has_equity {
            continue;
        }

        // Find the non-equity account and amount
        for entry in &tx.entries {
            if entry.account_path != "Equity:Uncategorized" {
                potential_transfers.push(PotentialTransfer {
                    transaction_id: tx.transaction.id,
                    description: tx.transaction.description.clone(),
                    amount: entry.amount,
                    date: tx.transaction.transaction_date,
                    non_equity_account: entry.account_path.clone(),
                    non_equity_account_id: entry.account_id,
                });
            }
        }
    }
    // Group by date and description to find matching transaction pairs
    let mut groups = std::collections::HashMap::new();
    for transfer in potential_transfers {
        let key = if allow_different_descriptions {
            // Group by date and amount only (ignore description differences)
            (
                transfer.date.date_naive(),
                "".to_string(),
                transfer.amount.abs(),
            )
        } else {
            // Group by date and description (exact match required)
            (
                transfer.date.date_naive(),
                transfer.description.clone(),
                Decimal::ZERO,
            )
        };
        groups.entry(key).or_insert_with(Vec::new).push(transfer);
    } // Filter groups with exactly 2 transfers (potential internal transfers)
    let transfer_groups: Vec<TransferGroup> = groups
        .into_iter()
        .filter_map(|((_date, _description, _amount), transfers)| {
            if transfers.len() == 2 {
                // Check if one is positive and one is negative (opposite movements)
                let positive_count = transfers
                    .iter()
                    .filter(|t| t.amount > Decimal::ZERO)
                    .count();
                let negative_count = transfers
                    .iter()
                    .filter(|t| t.amount < Decimal::ZERO)
                    .count();

                // Check if the amounts are opposite (one +X, one -X)
                let amount1 = transfers[0].amount;
                let amount2 = transfers[1].amount;

                if positive_count == 1 && negative_count == 1 && amount1 == -amount2 {
                    Some(TransferGroup {
                        date: transfers[0].date,
                        amount: amount1.abs(),
                        transfers,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    Ok(transfer_groups)
}

fn display_transfer_group(group_num: usize, group: &TransferGroup) {
    println!(
        "📋 Group {}: {} - Amount: €{:.2}",
        group_num,
        group.date.format("%Y-%m-%d"),
        group.amount
    );

    for (i, transfer) in group.transfers.iter().enumerate() {
        let sign = if transfer.amount > Decimal::ZERO {
            "+"
        } else {
            ""
        };
        println!(
            "  {}. {} → {} ({}€{:.2})",
            i + 1,
            truncate_string(&transfer.description, 40),
            truncate_string(&transfer.non_equity_account, 30),
            sign,
            transfer.amount
        );
    }
    println!();
}

async fn merge_transfer_group(
    transaction_service: &TransactionService,
    group: &TransferGroup,
) -> Result<Uuid> {
    // Find the source (negative amount) and destination (positive amount) accounts
    let source = group
        .transfers
        .iter()
        .find(|t| t.amount < Decimal::ZERO)
        .ok_or_else(|| anyhow::anyhow!("No source account found"))?;
    let destination = group
        .transfers
        .iter()
        .find(|t| t.amount > Decimal::ZERO)
        .ok_or_else(|| anyhow::anyhow!("No destination account found"))?;

    // Create the merged transaction description
    let new_description = format!(
        "Internal Transfer: {} → {}",
        source
            .non_equity_account
            .split(':')
            .next_back()
            .unwrap_or(&source.non_equity_account),
        destination
            .non_equity_account
            .split(':')
            .next_back()
            .unwrap_or(&destination.non_equity_account)
    );

    // Create a new internal transfer transaction
    let new_transaction = TransactionService::create_simple_transaction(
        new_description,
        destination.non_equity_account_id, // debit (destination receives money)
        source.non_equity_account_id,      // credit (source sends money)
        group.amount,
        group.date,
        None,
    );

    // Create the transaction in the database
    let created_transaction = transaction_service
        .create_transaction(new_transaction)
        .await?;
    let new_transaction_id = created_transaction.transaction.id;

    // Delete the original transactions
    for transfer in &group.transfers {
        transaction_service
            .delete_transaction(transfer.transaction_id)
            .await?;
    }

    Ok(new_transaction_id)
}
