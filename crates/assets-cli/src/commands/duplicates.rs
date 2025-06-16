use anyhow::Result;
use assets_core::{Database, DeduplicationService, MatchStatus, MatchType};
use clap::{Args, Subcommand};
use comfy_table::{presets::UTF8_FULL, Table};
use uuid::Uuid;

#[derive(Subcommand)]
pub enum DuplicateCommands {
    /// Find potential duplicate transactions
    Find(FindDuplicatesArgs),
    /// List all transactions with duplicate information
    List(ListDuplicatesArgs),
    /// Show details about potential duplicates for a transaction
    Show(ShowDuplicatesArgs),
    /// Confirm a duplicate match
    Confirm(ConfirmDuplicateArgs),
    /// Reject a duplicate match
    Reject(RejectDuplicateArgs),
    /// Run automatic duplicate detection on recent imports
    Detect(DetectDuplicatesArgs),
    /// Merge transactions (hide duplicate)
    Merge(MergeDuplicateArgs),
    /// Unmerge transaction (unhide duplicate)
    Unmerge(UnmergeDuplicateArgs),
}

#[derive(Args)]
pub struct FindDuplicatesArgs {
    /// Transaction ID to find duplicates for
    #[arg(short, long)]
    transaction_id: String,
    /// Amount tolerance for matching (default: 0.01)
    #[arg(long, default_value = "0.01")]
    amount_tolerance: String,
    /// Date tolerance in days (default: 3)
    #[arg(long, default_value = "3")]
    date_tolerance: i32,
}

#[derive(Args)]
pub struct ListDuplicatesArgs {
    /// Only show transactions that have potential duplicates
    #[arg(short, long)]
    only_duplicates: bool,
    /// Maximum number of transactions to show
    #[arg(short, long, default_value = "50")]
    limit: i32,
}

#[derive(Args)]
pub struct ShowDuplicatesArgs {
    /// Transaction ID to show duplicate information for
    #[arg(short, long)]
    transaction_id: String,
}

#[derive(Args)]
pub struct ConfirmDuplicateArgs {
    /// Transaction match ID to confirm
    #[arg(short, long)]
    match_id: String,
}

#[derive(Args)]
pub struct RejectDuplicateArgs {
    /// Transaction match ID to reject
    #[arg(short, long)]
    match_id: String,
}

#[derive(Args)]
pub struct DetectDuplicatesArgs {
    /// Import batch ID to detect duplicates for
    #[arg(short, long)]
    batch_id: String,
    /// Automatically confirm exact matches
    #[arg(long)]
    auto_confirm_exact: bool,
}

#[derive(Args)]
pub struct MergeDuplicateArgs {
    /// Primary transaction ID (the one to keep)
    #[arg(short, long)]
    primary_id: String,
    /// Duplicate transaction ID (the one to hide)
    #[arg(short, long)]
    duplicate_id: String,
}

#[derive(Args)]
pub struct UnmergeDuplicateArgs {
    /// Transaction ID to unhide
    #[arg(short, long)]
    transaction_id: String,
}

pub async fn handle_duplicate_command(command: DuplicateCommands) -> Result<()> {
    match command {
        DuplicateCommands::Find(args) => find_duplicates(args).await,
        DuplicateCommands::List(args) => list_duplicates(args).await,
        DuplicateCommands::Show(args) => show_duplicates(args).await,
        DuplicateCommands::Confirm(args) => confirm_duplicate(args).await,
        DuplicateCommands::Reject(args) => reject_duplicate(args).await,
        DuplicateCommands::Detect(args) => detect_duplicates(args).await,
        DuplicateCommands::Merge(args) => merge_duplicate(args).await,
        DuplicateCommands::Unmerge(args) => unmerge_duplicate(args).await,
    }
}

async fn find_duplicates(args: FindDuplicatesArgs) -> Result<()> {
    println!("🔍 Finding Potential Duplicates");
    println!("================================\n");

    let db = Database::from_env().await?;
    let dedup_service = DeduplicationService::new(db.pool().clone());

    let transaction_id = Uuid::parse_str(&args.transaction_id)?;
    let amount_tolerance = args.amount_tolerance.parse()?;

    let duplicates = dedup_service
        .find_potential_duplicates(
            transaction_id,
            Some(amount_tolerance),
            Some(args.date_tolerance),
        )
        .await?;

    if duplicates.is_empty() {
        println!(
            "✅ No potential duplicates found for transaction {}",
            args.transaction_id
        );
        return Ok(());
    }

    println!("Found {} potential duplicate(s):\n", duplicates.len());

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Transaction ID", "Confidence", "Match Criteria"]);

    for duplicate in &duplicates {
        table.add_row(vec![
            duplicate.potential_duplicate_id.to_string()[..8].to_string(),
            format!(
                "{:.2}%",
                duplicate.match_confidence * rust_decimal::Decimal::from(100)
            ),
            format!("{}", duplicate.match_criteria),
        ]);
    }

    println!("{table}");
    println!("\n💡 Use 'assets-cli duplicates show' to see details and confirm/reject matches");

    Ok(())
}

async fn list_duplicates(args: ListDuplicatesArgs) -> Result<()> {
    println!("📋 Transactions with Duplicate Information");
    println!("==========================================\n");

    let db = Database::from_env().await?;
    let dedup_service = DeduplicationService::new(db.pool().clone());

    let transactions = dedup_service
        .get_transactions_with_duplicates(Some(args.limit), args.only_duplicates)
        .await?;

    if transactions.is_empty() {
        if args.only_duplicates {
            println!("✅ No transactions with duplicates found");
        } else {
            println!("📭 No transactions found");
        }
        return Ok(());
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        "Date",
        "Description",
        "Amount",
        "Source",
        "Duplicates",
        "ID",
    ]);

    for tx in &transactions {
        let source = tx.import_source.as_deref().unwrap_or("-");
        let duplicate_indicator = if tx.has_duplicates { "⚠️" } else { "✅" };

        table.add_row(vec![
            tx.transaction_date.format("%Y-%m-%d").to_string(),
            truncate_string(&tx.description, 30),
            format!("{:.2}", tx.amount),
            source.to_string(),
            format!("{} {}", duplicate_indicator, tx.duplicate_count),
            tx.id.to_string()[..8].to_string(),
        ]);
    }

    println!("{table}");

    if args.only_duplicates {
        println!(
            "\n⚠️ Showing {} transactions with potential duplicates",
            transactions.len()
        );
        println!("💡 Use 'assets-cli duplicates show -t <transaction_id>' for details");
    } else {
        println!(
            "\n📊 Showing {} transactions (duplicates marked with ⚠️)",
            transactions.len()
        );
    }

    Ok(())
}

async fn show_duplicates(args: ShowDuplicatesArgs) -> Result<()> {
    println!("🔍 Duplicate Information for Transaction");
    println!("=======================================\n");

    let db = Database::from_env().await?;
    let dedup_service = DeduplicationService::new(db.pool().clone());

    // Try to parse as full UUID first, if that fails, look up by partial UUID
    let transaction_id = if let Ok(uuid) = Uuid::parse_str(&args.transaction_id) {
        uuid
    } else {
        // Look up transaction by partial UUID using the service
        match dedup_service
            .find_transaction_by_partial_uuid(&args.transaction_id)
            .await?
        {
            Some(id) => id,
            None => {
                println!(
                    "❌ No transaction found with ID starting with '{}'",
                    args.transaction_id
                );
                return Ok(());
            }
        }
    }; // Get the main transaction details
    let main_transaction = dedup_service
        .get_transaction_details_for_comparison(transaction_id)
        .await?;

    if let Some(main_tx) = main_transaction {
        println!("🎯 Main Transaction:");
        println!("   Date: {}", main_tx.transaction_date.format("%Y-%m-%d"));
        println!("   Description: {}", main_tx.description);
        println!(
            "   Source: {}",
            main_tx.import_source.as_deref().unwrap_or("N/A")
        );
        println!("   Accounts: {}", main_tx.entries_summary);
        println!("   ID: {}\n", main_tx.id);
    }

    let matches = dedup_service
        .get_matches_for_transaction(transaction_id)
        .await?;

    if matches.is_empty() {
        println!(
            "✅ No duplicate matches found for transaction {}",
            args.transaction_id
        );
        return Ok(());
    }

    println!("🔄 Found {} potential duplicate(s):\n", matches.len());

    for (index, match_record) in matches.iter().enumerate() {
        let other_tx_id = if match_record.primary_transaction_id == transaction_id {
            match_record.duplicate_transaction_id
        } else {
            match_record.primary_transaction_id
        };

        // Get details of the other transaction
        if let Some(other_tx) = dedup_service
            .get_transaction_details_for_comparison(other_tx_id)
            .await?
        {
            println!(
                "📋 Match #{} ({}% confidence, {:?}, {:?}):",
                index + 1,
                (match_record.match_confidence * rust_decimal::Decimal::from(100)).round(),
                match_record.match_type,
                match_record.status
            );
            println!("   Date: {}", other_tx.transaction_date.format("%Y-%m-%d"));
            println!("   Description: {}", other_tx.description);
            println!(
                "   Source: {}",
                other_tx.import_source.as_deref().unwrap_or("N/A")
            );
            println!("   Accounts: {}", other_tx.entries_summary);
            println!("   ID: {}", other_tx.id);
            println!(
                "   Match ID: {} (use this to confirm/reject)\n",
                &match_record.id.to_string()[..8]
            );
        }
    }

    println!("💡 Commands:");
    println!("   Confirm: assets-cli duplicates confirm -m <match_id>");
    println!("   Reject:  assets-cli duplicates reject -m <match_id>");

    Ok(())
}

async fn confirm_duplicate(args: ConfirmDuplicateArgs) -> Result<()> {
    println!("✅ Confirming Duplicate Match");
    println!("============================\n");

    let db = Database::from_env().await?;
    let dedup_service = DeduplicationService::new(db.pool().clone());

    // Try to parse as full UUID first, if that fails, look up by partial UUID
    let match_id = if let Ok(uuid) = Uuid::parse_str(&args.match_id) {
        uuid
    } else {
        match dedup_service
            .find_match_by_partial_uuid(&args.match_id)
            .await?
        {
            Some(id) => id,
            None => {
                println!(
                    "❌ No match found with ID starting with '{}'",
                    args.match_id
                );
                return Ok(());
            }
        }
    };

    let updated_match = dedup_service
        .update_match_status(match_id, MatchStatus::Confirmed)
        .await?;

    println!("✅ Match {} confirmed successfully", args.match_id);
    println!(
        "   Primary Transaction: {}",
        updated_match.primary_transaction_id
    );
    println!(
        "   Duplicate Transaction: {}",
        updated_match.duplicate_transaction_id
    );
    println!(
        "   Confidence: {:.1}%",
        updated_match.match_confidence * rust_decimal::Decimal::from(100)
    );

    Ok(())
}

async fn reject_duplicate(args: RejectDuplicateArgs) -> Result<()> {
    println!("❌ Rejecting Duplicate Match");
    println!("===========================\n");

    let db = Database::from_env().await?;
    let dedup_service = DeduplicationService::new(db.pool().clone());

    // Try to parse as full UUID first, if that fails, look up by partial UUID
    let match_id = if let Ok(uuid) = Uuid::parse_str(&args.match_id) {
        uuid
    } else {
        match dedup_service
            .find_match_by_partial_uuid(&args.match_id)
            .await?
        {
            Some(id) => id,
            None => {
                println!(
                    "❌ No match found with ID starting with '{}'",
                    args.match_id
                );
                return Ok(());
            }
        }
    };

    let updated_match = dedup_service
        .update_match_status(match_id, MatchStatus::Rejected)
        .await?;

    println!("❌ Match {} rejected successfully", args.match_id);
    println!(
        "   Primary Transaction: {}",
        updated_match.primary_transaction_id
    );
    println!(
        "   Duplicate Transaction: {}",
        updated_match.duplicate_transaction_id
    );
    println!(
        "   Confidence: {:.1}%",
        updated_match.match_confidence * rust_decimal::Decimal::from(100)
    );

    Ok(())
}

async fn detect_duplicates(args: DetectDuplicatesArgs) -> Result<()> {
    println!("🤖 Running Automatic Duplicate Detection");
    println!("========================================\n");

    let db = Database::from_env().await?;
    let dedup_service = DeduplicationService::new(db.pool().clone());

    let batch_id = Uuid::parse_str(&args.batch_id)?;
    let matches = dedup_service
        .detect_duplicates_for_batch(batch_id, args.auto_confirm_exact)
        .await?;

    if matches.is_empty() {
        println!("✅ No duplicates detected for batch {}", args.batch_id);
        return Ok(());
    }

    println!("🔍 Detected {} potential duplicate(s):\n", matches.len());

    let mut exact_count = 0;
    let mut probable_count = 0;
    let mut possible_count = 0;
    let mut confirmed_count = 0;

    for match_record in &matches {
        match match_record.match_type {
            MatchType::Exact => exact_count += 1,
            MatchType::Probable => probable_count += 1,
            MatchType::Possible => possible_count += 1,
        }

        if matches!(match_record.status, MatchStatus::Confirmed) {
            confirmed_count += 1;
        }
    }

    println!("📊 Summary:");
    println!(
        "   Exact matches: {} ({})",
        exact_count,
        if args.auto_confirm_exact {
            "auto-confirmed"
        } else {
            "pending review"
        }
    );
    println!("   Probable matches: {} (pending review)", probable_count);
    println!("   Possible matches: {} (pending review)", possible_count);

    if confirmed_count > 0 {
        println!("   Auto-confirmed: {}", confirmed_count);
    }

    println!("\n💡 Use 'assets-cli duplicates list --only-duplicates' to review pending matches");

    Ok(())
}

async fn merge_duplicate(args: MergeDuplicateArgs) -> Result<()> {
    println!("🔗 Merging Transactions");
    println!("========================\n");

    let db = Database::from_env().await?;
    let dedup_service = DeduplicationService::new(db.pool().clone());

    let primary_id = Uuid::parse_str(&args.primary_id)?;
    let duplicate_id = Uuid::parse_str(&args.duplicate_id)?;
    dedup_service
        .merge_transaction(primary_id, duplicate_id)
        .await?;

    println!("✅ Transaction {} merged successfully", args.primary_id);

    Ok(())
}

async fn unmerge_duplicate(args: UnmergeDuplicateArgs) -> Result<()> {
    println!("🔓 Unmerging Transaction");
    println!("========================\n");

    let db = Database::from_env().await?;
    let dedup_service = DeduplicationService::new(db.pool().clone());

    let transaction_id = Uuid::parse_str(&args.transaction_id)?;
    dedup_service.unmerge_transaction(transaction_id).await?;

    println!(
        "✅ Transaction {} unmerged successfully",
        args.transaction_id
    );

    Ok(())
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    } else {
        s.to_string()
    }
}
