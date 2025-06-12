use crate::error::Result;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};

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
}
