use crate::error::Result;
use crate::importers::{ImportedPayslip, PayslipImporter};
use crate::models::{NewJournalEntry, NewTransaction};
use crate::services::{AccountService, TransactionService};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PayslipImportService {
    transaction_service: TransactionService,
    account_service: AccountService,
}

pub struct DestinationAccount {
    pub fixed_gross: String,          // e.g., "Income:Salary:Fixed"
    pub variable_gross: String,       // e.g., "Income:Salary:Variable"
    pub net_pay: String,              // e.g., "Assets:Current Assets:Checking"
    pub social_contributions: String, // e.g., "Expenses:Social Contributions"
    pub revenue_taxes: String,        // e.g., "Expenses:Revenue Taxes"
    pub meal_vouchers: String,        // e.g., "Assets:Meal Vouchers"
    pub meal_vouchers_income: String, // e.g., "Income:Meal Vouchers"
    pub additional_benefits: String,  // e.g., "Income:Additional Benefits"
}

impl PayslipImportService {
    pub fn new(pool: PgPool) -> Self {
        let transaction_service = TransactionService::new(pool.clone());
        let account_service = AccountService::new(pool.clone());

        Self {
            transaction_service,
            account_service,
        }
    }

    /// Import a payslip using the specified importer and convert to transactions
    pub async fn import_payslip<T: PayslipImporter>(
        &self,
        importer: &T,
        file_path: &str,
        destination_account: &DestinationAccount,
    ) -> Result<ImportResult> {
        // Import the payslip data
        let payslip = importer.import_from_file(file_path).await?;

        // Convert payslip to transaction
        let transaction_id = self
            .create_payslip_transaction(&payslip, destination_account)
            .await?;

        Ok(ImportResult {
            payslip_info: payslip,
            transaction_id,
            accounts_created: vec![], // We'll populate this if we create any accounts
            warnings: vec![],
        })
    }

    /// Create a double-entry transaction from a payslip
    async fn create_payslip_transaction(
        &self,
        payslip: &ImportedPayslip,
        destination_account: &DestinationAccount,
    ) -> Result<Uuid> {
        let entries = [
            (
                &destination_account.fixed_gross,
                -payslip.gross_fixed_salary,
            ),
            (
                &destination_account.variable_gross,
                -payslip.gross_variable_salary.values().sum::<Decimal>(),
            ),
            (
                &destination_account.social_contributions,
                payslip.total_social_contributions,
            ),
            (
                &destination_account.revenue_taxes,
                payslip.total_revenue_taxes,
            ),
            (
                &destination_account.meal_vouchers,
                payslip.meal_vouchers_employee_contribution
                    + payslip.meal_vouchers_employer_contribution,
            ),
            (
                &destination_account.meal_vouchers_income,
                -payslip.meal_vouchers_employer_contribution,
            ),
            (
                &destination_account.additional_benefits,
                -payslip.additional_benefits.values().sum::<Decimal>(),
            ),
            (&destination_account.net_pay, payslip.net_paid_salary),
        ];

        let journal_entry_futures = entries.iter().map(|(path, amount)| {
            let account_service = &self.account_service;
            async move {
                let account = account_service
                    .get_account_by_path(path)
                    .await
                    .map_err(|_| {
                        // Account not found, create it
                        crate::error::CoreError::NotFound(format!(
                            "Account not found: {}. Please create this account first.",
                            path
                        ))
                    })?;
                Ok::<_, crate::error::CoreError>(NewJournalEntry {
                    account_id: account.id,
                    amount: *amount,
                    memo: None, // Memo can be added later if needed
                })
            }
        });

        let journal_entries: Vec<NewJournalEntry> =
            futures::future::try_join_all(journal_entry_futures).await?;

        let transaction_request = NewTransaction {
            description: format!(
                "Payslip - {} ({})",
                payslip.employer_name,
                payslip.pay_date.format("%Y-%m-%d")
            ),
            reference: Some(format!(
                "PAYSLIP-{}-{}",
                payslip.pay_date.format("%Y%m%d"),
                payslip.employer_name.replace(" ", "")
            )),
            transaction_date: payslip.pay_date.and_hms_opt(12, 0, 0).unwrap().and_utc(),
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
    pub payslip_info: ImportedPayslip,
    pub transaction_id: Uuid,
    pub accounts_created: Vec<String>,
    pub warnings: Vec<String>,
}
