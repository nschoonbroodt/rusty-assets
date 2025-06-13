use anyhow::Result;
use assets_core::{Database, TransactionService, TransactionWithEntriesAndAccounts};
use chrono::{NaiveDate, DateTime, Utc};
use clap::{Args, Subcommand};
use comfy_table::{presets::UTF8_FULL, Table};
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Subcommand)]
pub enum TransactionCommands {
    /// List transactions with optional filtering
    List(ListTransactionsArgs),
    /// Show detailed view of a specific transaction
    Show {
        /// Transaction ID to show
        id: String,
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
    
    /// Filter by user ID
    #[arg(long)]
    user_id: Option<String>,
    
    /// Maximum number of transactions to show
    #[arg(long, default_value = "50")]
    limit: u32,
    
    /// Output format
    #[arg(long, value_enum, default_value = "table")]
    format: OutputFormat,
}

#[derive(clap::ValueEnum, Clone)]
enum OutputFormat {
    Table,
    Json,
    Csv,
}

pub async fn handle_transaction_command(command: TransactionCommands) -> Result<()> {
    match command {
        TransactionCommands::List(args) => list_transactions(args).await,
        TransactionCommands::Show { id } => show_transaction(&id).await,
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
    
    let user_id = if let Some(user_str) = &args.user_id {
        Some(Uuid::parse_str(user_str)?)
    } else {
        None
    };
      // Get transactions with filters
    let transactions = transaction_service
        .get_transactions_with_filters_and_accounts(from_date, to_date, args.account.as_deref(), user_id, args.limit)
        .await?;
    
    if transactions.is_empty() {
        println!("No transactions found with the specified filters.");
        println!();
        println!("💡 Try adjusting your filters or check:");
        println!("   - Date range with --from and --to");
        println!("   - Account filter with --account");
        println!("   - User filter with --user-id");
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
      match transaction_service.get_transaction_with_accounts(transaction_id).await? {
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
        "Date", "Description", "Entries", "Amount", "Reference", "ID"
    ]);

    for tx in transactions {
        let total_amount: Decimal = tx.entries.iter().map(|e| e.amount.abs()).sum::<Decimal>() / Decimal::from(2); // Divide by 2 since double-entry
        let entry_count = tx.entries.len();
        let reference = tx.transaction.reference.as_deref().unwrap_or("-");
        
        table.add_row(vec![
            tx.transaction.transaction_date.format("%Y-%m-%d").to_string(),
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
        let total_amount: Decimal = tx.entries.iter().map(|e| e.amount.abs()).sum::<Decimal>() / Decimal::from(2);
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
    if let Some(created_by) = tx.created_by {
        println!("   Created by: {}", created_by);
    }
    println!("   Created at: {}", tx.created_at.format("%Y-%m-%d %H:%M"));
    
    println!();
    println!("📊 Journal Entries:");
    
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Account Path", "Account Name", "Amount", "Type", "Memo"]);
    
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
    let total: Decimal = transaction_with_entries.entries.iter().map(|e| e.amount).sum();
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
