use crate::error::Result;
use crate::importers::{ImportedTransaction, TransactionImporter};
use crate::services::{AccountService, TransactionService};
use rust_decimal::Decimal;
use uuid::Uuid;

pub struct ImportService {
    account_service: AccountService,
    transaction_service: TransactionService,
}

impl ImportService {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self {
            account_service: AccountService::new(db.clone()),
            transaction_service: TransactionService::new(db),
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
        println!("📁 Importing from: {}", file_path);
        println!("🏦 Target account: {}", target_account_path);

        // Verify target account exists
        let target_account = self
            .account_service
            .get_account_by_path(target_account_path)
            .await?; // Import raw transactions
        let imported = importer.import_from_file(file_path).await?;
        println!("📊 Found {} transactions", imported.len());

        let mut created_count = 0;
        let mut skipped_count = 0;
        let mut errors = Vec::new();
        let total_count = imported.len();

        for imported_tx in imported {
            match self
                .create_transaction_from_import(&imported_tx, &target_account.id, user_id)
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
            let abs_amount = imported.amount.abs();
            TransactionService::create_simple_transaction(
                imported.description.clone(),
                expense_account_id, // debit (expense)
                card_account_id,    // credit (increase liability)
                abs_amount,
                transaction_date,
                None,
                Some(user_id),
            )
        } else if self.is_card_settlement_transaction(&imported.description) {
            // Handle monthly card settlement
            let card_account_id = self.get_or_create_deferred_card_account().await?;

            // Card settlement: Card liability account (debit) / Bank account (credit)
            let abs_amount = imported.amount.abs();
            TransactionService::create_simple_transaction(
                imported.description.clone(),
                card_account_id,    // debit (reduce liability)
                *target_account_id, // credit (money out of bank)
                abs_amount,
                transaction_date,
                None,
                Some(user_id),
            )
        } else {
            // Handle regular transactions (not card-related)
            let other_account_id = self.determine_other_account(imported).await?;

            if imported.amount > Decimal::ZERO {
                // Money coming in: debit target account, credit other account
                TransactionService::create_simple_transaction(
                    imported.description.clone(),
                    *target_account_id, // debit (money in)
                    other_account_id,   // credit (income source)
                    imported.amount,
                    transaction_date,
                    None,
                    Some(user_id),
                )
            } else {
                // Money going out: debit other account, credit target account
                let abs_amount = imported.amount.abs();
                TransactionService::create_simple_transaction(
                    imported.description.clone(),
                    other_account_id,   // debit (expense)
                    *target_account_id, // credit (money out)
                    abs_amount,
                    transaction_date,
                    None,
                    Some(user_id),
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
        // Handle deferred debit card transactions
        if self.is_card_transaction(&imported.description) {
            return self.get_or_create_deferred_card_account().await;
        }

        // Handle monthly card settlement transactions
        if self.is_card_settlement_transaction(&imported.description) {
            return self.get_or_create_deferred_card_account().await;
        }

        // Handle internal transfers (VIR transactions) - these go to other accounts
        if self.is_internal_transfer(&imported.description) {
            return self.get_or_create_transfers_pending_account().await;
        }

        // Map BoursoBank categories to our account structure
        let account_path = match (
            &imported.category_parent,
            &imported.category,
            imported.amount > Decimal::ZERO,
        ) {
            // Income categories
            (Some(parent), _, true) if parent.contains("Virements reçus") => "Income:Salary",
            (Some(parent), _, true) if parent.contains("Revenus d'épargne") => "Income:Investment",

            // Expense categories based on BoursoBank categorization
            (Some(parent), Some(category), false) => match parent.as_str() {
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
                "Logement" => "Expenses:Housing:Mortgage",
                "Services financiers & professionnels" => "Expenses:Financial:Fees",
                "Dépenses d'épargne" => "Assets:Savings:Insurance",
                "Mouvements internes débiteurs" => "Assets:Transfers:Pending",
                _ => "Expenses:Uncategorized",
            },

            // Fallback
            (_, _, true) => "Income:Other",
            (_, _, false) => "Expenses:Uncategorized",
        };

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
        description.starts_with("CARTE ") && 
        // Exclude the monthly settlement transaction
        !description.contains("Relevé différé Carte")
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
    /// Check if a transaction description indicates an internal transfer
    fn is_internal_transfer(&self, description: &str) -> bool {
        // Only catch actual internal transfers between your own accounts
        description.contains("Virement interne") 
            || description.contains("Versement initial")
            || description.starts_with("VIR INST")  // VIR INST = internal institutional transfer
            || (description.starts_with("VIR ") && description.contains("depuis")) // VIR depuis = from another account
    }

    /// Get or create the transfers pending account for internal transfers
    async fn get_or_create_transfers_pending_account(&self) -> Result<Uuid> {
        let transfers_account_path = "Assets:Transfers:Pending";

        match self
            .account_service
            .get_account_by_path(transfers_account_path)
            .await
        {
            Ok(account) => Ok(account.id),
            Err(_) => {
                // Account doesn't exist, create it or use fallback
                println!(
                    "⚠️  Transfers account '{}' doesn't exist. Using fallback.",
                    transfers_account_path
                );
                let fallback_account = self
                    .account_service
                    .get_account_by_path("Expenses:Uncategorized")
                    .await?;
                Ok(fallback_account.id)
            }
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
