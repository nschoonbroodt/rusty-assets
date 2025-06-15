use crate::error::Result;
use crate::importers::{ImportedPayslip, PayslipImporter};
use crate::models::{NewJournalEntry, NewTransaction};
use crate::services::{AccountService, TransactionService, UserService};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PayslipImportService {
    transaction_service: TransactionService,
    account_service: AccountService,
    user_service: UserService,
}

impl PayslipImportService {
    pub fn new(pool: PgPool) -> Self {
        let transaction_service = TransactionService::new(pool.clone());
        let account_service = AccountService::new(pool.clone());
        let user_service = UserService::new(pool.clone());

        Self {
            transaction_service,
            account_service,
            user_service,
        }
    }

    /// Import a payslip using the specified importer and convert to transactions
    pub async fn import_payslip<T: PayslipImporter>(
        &self,
        importer: &T,
        file_path: &str,
        destination_account_path: &str,
        user_name: &str,
    ) -> Result<ImportResult> {
        // Import the payslip data
        let payslip = importer.import_from_file(file_path).await?;

        // Get the user
        let user = self
            .user_service
            .get_user_by_name(user_name)
            .await?
            .ok_or_else(|| {
                crate::error::CoreError::NotFound(format!("User not found: {}", user_name))
            })?;

        // Convert payslip to transaction
        let transaction_id = self
            .create_payslip_transaction(&payslip, destination_account_path, user.id)
            .await?;

        Ok(ImportResult {
            payslip_info: PayslipInfo {
                pay_date: payslip.pay_date,
                pay_period_start: payslip.pay_period_start,
                pay_period_end: payslip.pay_period_end,
                employee_name: payslip.employee_name,
                employer_name: payslip.employer_name,
                gross_salary: payslip.gross_salary,
                net_salary: payslip.net_salary,
                line_items_count: payslip.line_items.len(),
            },
            transaction_id,
            accounts_created: vec![], // We'll populate this if we create any accounts
            warnings: vec![],
        })
    }

    /// Create a double-entry transaction from a payslip
    async fn create_payslip_transaction(
        &self,
        payslip: &ImportedPayslip,
        destination_account_path: &str,
        user_id: Uuid,
    ) -> Result<Uuid> {
        let mut journal_entries = Vec::new(); // Ensure destination account exists (the checking account where net pay goes)
        let destination_account = self
            .account_service
            .get_account_by_path(destination_account_path)
            .await?; // Create journal entries for each payslip line item (except NetPay which is handled separately)
        for line_item in &payslip.line_items {
            // Skip NetPay line items as they are handled by the destination account
            if matches!(
                line_item.item_type,
                crate::importers::PayslipItemType::NetPay
            ) {
                continue;
            }

            let account_path = line_item
                .account_mapping
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or(line_item.item_type.suggested_account_path()); // Get or create the account for this line item
            let account = self
                .account_service
                .get_account_by_path(account_path)
                .await
                .or_else(|_| {
                    // Account not found
                    Err(crate::error::CoreError::NotFound(format!(
                        "Account not found: {}. Please create this account first.",
                        account_path
                    )))
                })?; // Determine debit/credit based on account type and item type
            let amount = match account.account_type {
                crate::models::AccountType::Income => {
                    // Income accounts are credited (increased) by income items
                    -line_item.amount // Negative amount for credit in our system
                }
                crate::models::AccountType::Expense => {
                    // Expense accounts are debited (increased) by expense items
                    line_item.amount // Positive amount for debit
                }
                crate::models::AccountType::Asset => {
                    // This shouldn't happen for payslip line items, but handle gracefully
                    line_item.amount
                }
                _ => {
                    return Err(crate::error::CoreError::ValidationError(format!(
                        "Unexpected account type for payslip item: {:?} (account: {})",
                        account.account_type, account_path
                    )));
                }
            };
            journal_entries.push(NewJournalEntry {
                account_id: account.id,
                amount,
                memo: Some(line_item.description.clone()),
            });
        } // Add the destination account entry (net salary to checking account)
        journal_entries.push(NewJournalEntry {
            account_id: destination_account.id,
            amount: payslip.net_salary,
            memo: Some(format!("Net salary - {}", payslip.employer_name)),
        }); // Create the transaction
        let transaction_request = NewTransaction {
            description: format!(
                "Payslip - {} ({} to {})",
                payslip.employer_name,
                payslip.pay_period_start.format("%Y-%m-%d"),
                payslip.pay_period_end.format("%Y-%m-%d")
            ),
            reference: Some(format!(
                "PAYSLIP-{}-{}",
                payslip.pay_date.format("%Y%m%d"),
                payslip.employer_name.replace(" ", "")
            )),
            transaction_date: payslip.pay_date.and_hms_opt(12, 0, 0).unwrap().and_utc(),
            created_by: Some(user_id),
            entries: journal_entries,
            import_source: Some("Payslip".to_string()),
            import_batch_id: None, // Payslips are imported individually
            external_reference: Some(format!("PAYSLIP-{}", payslip.pay_date.format("%Y%m%d"))),
        };

        let result = self
            .transaction_service
            .create_transaction(transaction_request)
            .await?;

        Ok(result.transaction.id)
    }
}

#[derive(Debug)]
pub struct ImportResult {
    pub payslip_info: PayslipInfo,
    pub transaction_id: Uuid,
    pub accounts_created: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug)]
pub struct PayslipInfo {
    pub pay_date: NaiveDate,
    pub pay_period_start: NaiveDate,
    pub pay_period_end: NaiveDate,
    pub employee_name: String,
    pub employer_name: String,
    pub gross_salary: Decimal,
    pub net_salary: Decimal,
    pub line_items_count: usize,
}
