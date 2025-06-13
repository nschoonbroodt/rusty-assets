use crate::error::Result;
use crate::models::{AccountLedgerRow, IncomeStatementRow};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug)]
pub struct BalanceSheetData {
    pub assets: Vec<AccountBalance>,
    pub liabilities: Vec<AccountBalance>,
    pub equity: Vec<AccountBalance>,
    pub total_assets: Decimal,
    pub total_liabilities: Decimal,
    pub total_equity: Decimal,
    pub report_date: NaiveDate,
}

#[derive(Debug)]
pub struct AccountBalance {
    pub name: String,
    pub full_path: String,
    pub balance: Decimal,
    pub level: i32,
}

pub struct ReportService {
    pool: PgPool,
}
impl ReportService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn balance_sheet(&self, report_date: NaiveDate) -> Result<BalanceSheetData> {
        let rows = sqlx::query(
            "SELECT account_type, name, balance, full_path, level FROM balance_sheet_data($1)",
        )
        .bind(report_date)
        .fetch_all(&self.pool)
        .await?;

        let mut assets = Vec::new();
        let mut liabilities = Vec::new();
        let mut equity = Vec::new();
        let mut total_assets = Decimal::ZERO;
        let mut total_liabilities = Decimal::ZERO;
        let mut total_equity = Decimal::ZERO;

        for row in rows {
            let account_type: String = row.get("account_type");
            let balance: Decimal = row.get("balance");

            let account_balance = AccountBalance {
                name: row.get("name"),
                full_path: row.get("full_path"),
                balance,
                level: row.get("level"),
            };

            match account_type.as_str() {
                "asset" => {
                    total_assets += balance;
                    assets.push(account_balance);
                }
                "liability" => {
                    total_liabilities += balance;
                    liabilities.push(account_balance);
                }
                "equity" => {
                    total_equity += balance;
                    equity.push(account_balance);
                }
                _ => {} // Skip unknown types, income and expense accounts
            }
        }

        Ok(BalanceSheetData {
            assets,
            liabilities,
            equity,
            total_assets,
            total_liabilities,
            total_equity,
            report_date,
        })
    }

    pub async fn income_statement(
        &self,
        start_date: NaiveDate, // Reordered and changed user_ids to user_id
        end_date: NaiveDate,
        user_id: Uuid, // Changed from &[Uuid] to Uuid
    ) -> Result<Vec<IncomeStatementRow>> {
        // Convert single Uuid to a slice for the SQL query
        let user_ids_array = [user_id];
        let rows = sqlx::query_as::<_, IncomeStatementRow>(
            "SELECT category_name, account_name, account_path, total_amount FROM fn_income_statement($1, $2, $3)", // Updated to include account_path
        )
        .bind(&user_ids_array) // Bind as a slice
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn account_ledger(
        &self,
        account_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<AccountLedgerRow>> {
        let rows = sqlx::query_as::<_, AccountLedgerRow>(
            "SELECT transaction_date, transaction_id, description, reference, memo, debit_amount, credit_amount, running_balance FROM fn_account_ledger($1, $2, $3)",
        )
        .bind(account_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
}
