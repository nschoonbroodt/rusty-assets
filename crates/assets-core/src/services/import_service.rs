use crate::error::Result;
use crate::importers::{ImportedTransaction, TransactionImporter};
use crate::services::{
    AccountService, DeduplicationService, FileImportService, TransactionService,
};
use rust_decimal::Decimal;
use uuid::Uuid;

pub struct ImportService {
    account_service: AccountService,
    transaction_service: TransactionService,
    file_import_service: FileImportService,
    deduplication_service: DeduplicationService,
}

impl ImportService {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self {
            account_service: AccountService::new(db.clone()),
            transaction_service: TransactionService::new(db.clone()),
            file_import_service: FileImportService::new(db.clone()),
            deduplication_service: DeduplicationService::new(db),
        }
    }
    /// Import transactions using the specified importer
    pub async fn import_transactions<T: TransactionImporter>(
        &self,
        importer: &T,
        file_path: &str,
        target_account_path: &str,
        user_id: Uuid,
    ) -> Result<ImportSummary> {
        // Generate a batch ID for this import
        let import_batch_id = Uuid::new_v4();
        let import_source = self.get_import_source_name(importer);

        println!("📁 Importing from: {}", file_path);
        println!("🏦 Target account: {}", target_account_path);
        println!("📦 Import batch ID: {}", import_batch_id);
        println!("🔍 Import source: {}", import_source);

        // Check if file has already been imported
        let file_hash = FileImportService::calculate_file_hash(file_path)?;
        if self
            .file_import_service
            .is_file_already_imported(&file_hash)
            .await?
        {
            if let Some(existing_file) = self
                .file_import_service
                .get_imported_file_by_hash(&file_hash)
                .await?
            {
                return Err(crate::error::CoreError::ImportError(format!(
                    "File already imported on {} from source '{}'. {} transactions were imported. File: {}",
                    existing_file.imported_at.format("%Y-%m-%d %H:%M:%S"),
                    existing_file.import_source,
                    existing_file.transaction_count,
                    existing_file.file_name
                )));
            }
        }

        // Verify target account exists
        let target_account = self
            .account_service
            .get_account_by_path(target_account_path)
            .await?;

        // Import raw transactions
        let imported = importer.import_from_file(file_path).await?;
        println!("📊 Found {} transactions", imported.len());

        let mut created_count = 0;
        let mut skipped_count = 0;
        let mut errors = Vec::new();
        let total_count = imported.len();
        for imported_tx in imported {
            match self
                .create_transaction_from_import(
                    &imported_tx,
                    &target_account.id,
                    user_id,
                    import_batch_id,
                    &import_source,
                )
                .await
            {
                Ok(_) => {
                    created_count += 1;
                    if created_count % 10 == 0 {
                        println!("  ✅ Processed {} transactions...", created_count);
                    }
                }
                Err(e) => {
                    errors.push(format!("Transaction '{}': {}", imported_tx.description, e));
                    skipped_count += 1;
                }
            }
        }

        // Record the file import (only if we had successful imports)
        if created_count > 0 {
            let file_metadata = self.file_import_service.prepare_file_metadata(
                file_path,
                &import_source,
                import_batch_id,
                user_id,
                created_count as i32,
                Some(format!(
                    "Imported {} transactions, skipped {}",
                    created_count, skipped_count
                )),
            )?;
            self.file_import_service
                .record_file_import(file_metadata)
                .await?;
            println!("📝 File import recorded in database");

            // Automatically run duplicate detection on the imported batch
            println!("🔍 Running automatic duplicate detection...");
            match self
                .deduplication_service
                .detect_duplicates_for_batch(import_batch_id, true) // Auto-confirm exact matches
                .await
            {
                Ok(matches) => {
                    if !matches.is_empty() {
                        let exact_count = matches
                            .iter()
                            .filter(|m| {
                                matches!(
                                    m.match_type,
                                    crate::services::deduplication_service::MatchType::Exact
                                )
                            })
                            .count();
                        let probable_count = matches
                            .iter()
                            .filter(|m| {
                                matches!(
                                    m.match_type,
                                    crate::services::deduplication_service::MatchType::Probable
                                )
                            })
                            .count();
                        let possible_count = matches
                            .iter()
                            .filter(|m| {
                                matches!(
                                    m.match_type,
                                    crate::services::deduplication_service::MatchType::Possible
                                )
                            })
                            .count();

                        println!("🎯 Detected {} potential duplicate(s):", matches.len());
                        if exact_count > 0 {
                            println!("   📌 {} exact match(es) - auto-confirmed", exact_count);
                        }
                        if probable_count > 0 {
                            println!("   🟡 {} probable match(es) - needs review", probable_count);
                        }
                        if possible_count > 0 {
                            println!("   🟠 {} possible match(es) - needs review", possible_count);
                        }
                        println!("💡 Use 'assets-cli duplicates list --only-duplicates' to review");
                    } else {
                        println!("✅ No duplicates detected in this import");
                    }
                }
                Err(e) => {
                    println!("⚠️  Duplicate detection failed: {}", e);
                    println!(
                        "   Import was successful, but you may want to run 'assets-cli duplicates detect' manually"
                    );
                }
            }
        }

        Ok(ImportSummary {
            total: total_count,
            created: created_count,
            skipped: skipped_count,
            errors,
        })
    }
    async fn create_transaction_from_import(
        &self,
        imported: &ImportedTransaction,
        target_account_id: &Uuid,
        user_id: Uuid,
        import_batch_id: Uuid,
        import_source: &str,
    ) -> Result<Uuid> {
        // Convert naive date to DateTime<Utc>
        let transaction_date = imported.date_op.and_hms_opt(12, 0, 0).unwrap().and_utc();
        let new_transaction = if self.is_card_transaction(&imported.description) {
            // Handle deferred debit card transactions
            let card_account_id = self.get_or_create_deferred_card_account().await?;
            let expense_account_id = self
                .determine_expense_account_for_card_transaction(imported)
                .await?;

            // Card purchase: Expense account (debit) / Card liability account (credit)
            // The bank account is not immediately affected
            let amount = -imported.amount;
            TransactionService::create_simple_transaction_with_import(
                imported.description.clone(),
                expense_account_id, // debit (expense)
                card_account_id,    // credit (increase liability)
                amount,
                transaction_date,
                None,
                Some(user_id),
                Some(import_source.to_string()),
                Some(import_batch_id),
                Some(imported.description.clone()),
            )
        } else if self.is_card_settlement_transaction(&imported.description) {
            // Handle monthly card settlement
            let card_account_id = self.get_or_create_deferred_card_account().await?;

            // Card settlement: Card liability account (debit) / Bank account (credit)
            let amount = -imported.amount;
            TransactionService::create_simple_transaction_with_import(
                imported.description.clone(),
                card_account_id,    // debit (reduce liability)
                *target_account_id, // credit (money out of bank)
                amount,
                transaction_date,
                None,
                Some(user_id),
                Some(import_source.to_string()),
                Some(import_batch_id),
                Some(imported.description.clone()),
            )
        } else {
            // Handle regular transactions (not card-related)
            let other_account_id = self.determine_other_account(imported).await?;

            if imported.amount > Decimal::ZERO {
                // Money coming in: debit target account, credit other account
                TransactionService::create_simple_transaction_with_import(
                    imported.description.clone(),
                    *target_account_id, // debit (money in)
                    other_account_id,   // credit (income source)
                    imported.amount,
                    transaction_date,
                    None,
                    Some(user_id),
                    Some(import_source.to_string()),
                    Some(import_batch_id),
                    Some(imported.description.clone()),
                )
            } else {
                // Money going out: debit other account, credit target account
                let abs_amount = imported.amount.abs();
                TransactionService::create_simple_transaction_with_import(
                    imported.description.clone(),
                    other_account_id,   // debit (expense)
                    *target_account_id, // credit (money out)
                    abs_amount,
                    transaction_date,
                    None,
                    Some(user_id),
                    Some(import_source.to_string()),
                    Some(import_batch_id),
                    Some(imported.description.clone()),
                )
            }
        };

        let transaction_with_entries = self
            .transaction_service
            .create_transaction(new_transaction)
            .await?;
        Ok(transaction_with_entries.transaction.id)
    }
    async fn determine_other_account(&self, imported: &ImportedTransaction) -> Result<Uuid> {
        // Modified to put everything in the account
        let account_path = "Equity:Uncategorized";

        // Try to get the account, create if it doesn't exist
        match self.account_service.get_account_by_path(account_path).await {
            Ok(account) => Ok(account.id),
            Err(_) => {
                // Account doesn't exist, for now use a default
                // In the future, we could auto-create the account hierarchy
                let fallback_path = if imported.amount > Decimal::ZERO {
                    "Income:Other"
                } else {
                    "Expenses:Uncategorized"
                };
                let account = self
                    .account_service
                    .get_account_by_path(fallback_path)
                    .await?;
                Ok(account.id)
            }
        }
    }

    /// Determine the expense account for a card transaction based on BoursoBank categorization
    async fn determine_expense_account_for_card_transaction(
        &self,
        imported: &ImportedTransaction,
    ) -> Result<Uuid> {
        // Use the same logic as determine_other_account but only for expenses
        let account_path = match (&imported.category_parent, &imported.category) {
            (Some(parent), Some(category)) => match parent.as_str() {
                "Vie quotidienne" => match category.as_str() {
                    "Alimentation" => "Expenses:Food:Groceries",
                    "Vêtements et accessoires" => "Expenses:Personal:Clothing",
                    "Bricolage et jardinage" => "Expenses:Home:Maintenance",
                    "Equipements sportifs et artistiques" => "Expenses:Personal:Sports",
                    _ => "Expenses:Personal:Other",
                },
                "Loisirs et sorties" => match category.as_str() {
                    "Restaurants, bars, discothèques…" => "Expenses:Food:Restaurants",
                    _ => "Expenses:Entertainment",
                },
                "Voyages & Transports" => match category.as_str() {
                    "Hébergement (hôtels, camping…)" => "Expenses:Travel:Accommodation",
                    "Taxis" => "Expenses:Transportation:Taxi",
                    "Transports quotidiens (métro, bus…)" => "Expenses:Transportation:Public",
                    _ => "Expenses:Travel",
                },
                "Auto & Moto" => match category.as_str() {
                    "Carburant" => "Expenses:Transportation:Fuel",
                    "Parking" => "Expenses:Transportation:Parking",
                    "Péages" => "Expenses:Transportation:Tolls",
                    _ => "Expenses:Transportation:Other",
                },
                "Abonnements & téléphonie" => "Expenses:Utilities:Subscriptions",
                _ => "Expenses:Uncategorized",
            },
            _ => "Expenses:Uncategorized",
        };

        // Try to get the account, create if it doesn't exist
        match self.account_service.get_account_by_path(account_path).await {
            Ok(account) => Ok(account.id),
            Err(_) => {
                // Account doesn't exist, use fallback
                let account = self
                    .account_service
                    .get_account_by_path("Expenses:Uncategorized")
                    .await?;
                Ok(account.id)
            }
        }
    }

    /// Check if a transaction description indicates a deferred debit card transaction
    fn is_card_transaction(&self, description: &str) -> bool {
        (description.starts_with("CARTE ") && 
        // Exclude the monthly settlement transaction
        !description.contains("Relevé différé Carte"))
            || description.starts_with("AVOIR ")
            || description.starts_with("RETRAIT DAB ")
    }

    /// Check if a transaction description indicates a monthly card settlement
    fn is_card_settlement_transaction(&self, description: &str) -> bool {
        description.contains("Relevé différé Carte") || description.contains("Releve differe Carte")
    }

    /// Get or create the deferred debit card account
    async fn get_or_create_deferred_card_account(&self) -> Result<Uuid> {
        let card_account_path = "Liabilities:Current Liabilities:Deferred Debit Card";

        match self
            .account_service
            .get_account_by_path(card_account_path)
            .await
        {
            Ok(account) => Ok(account.id),
            Err(_) => {
                // Account doesn't exist, create it
                // For now, return a fallback account - in production you'd want to create the hierarchy
                println!(
                    "⚠️  Deferred card account '{}' doesn't exist. Using fallback.",
                    card_account_path
                );
                let fallback_account = self
                    .account_service
                    .get_account_by_path("Expenses:Uncategorized")
                    .await?;
                Ok(fallback_account.id)
            }
        }
    }

    /// Get a clean import source name from the importer type
    fn get_import_source_name<T: TransactionImporter>(&self, importer: &T) -> String {
        let description = importer.format_description();
        // Extract just the bank name from format description
        if description.contains("BoursoBank") {
            "BoursoBank".to_string()
        } else if description.contains("Société Générale") {
            "SocieteGenerale".to_string()
        } else {
            "Unknown".to_string()
        }
    }
}

#[derive(Debug)]
pub struct ImportSummary {
    pub total: usize,
    pub created: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

impl ImportSummary {
    pub fn print_summary(&self) {
        println!("\n📊 Import Summary:");
        println!("   Total transactions: {}", self.total);
        println!("   Created: ✅ {}", self.created);
        if self.skipped > 0 {
            println!("   Skipped: ⚠️ {}", self.skipped);
        }

        if !self.errors.is_empty() {
            println!("\n❌ Errors:");
            for error in &self.errors {
                println!("   {}", error);
            }
        }
    }
}
