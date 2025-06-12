use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use assets_core::{Database, ReportService};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct BalanceSheetQuery {
    #[serde(default)]
    include_zero: bool,
    #[serde(default = "default_format")]
    format: String,
}

fn default_format() -> String {
    "json".to_string()
}

#[derive(Debug, Serialize)]
pub struct BalanceSheetResponse {
    pub report_type: String,
    pub report_date: NaiveDate,
    pub currency: String,
    pub assets: Vec<AccountBalanceResponse>,
    pub liabilities: Vec<AccountBalanceResponse>,
    pub equity: Vec<AccountBalanceResponse>,
    pub totals: BalanceSheetTotals,
    pub metadata: BalanceSheetMetadata,
}

#[derive(Debug, Serialize)]
pub struct AccountBalanceResponse {
    pub name: String,
    pub full_path: String,
    pub balance: rust_decimal::Decimal,
    pub level: i32,
}

#[derive(Debug, Serialize)]
pub struct BalanceSheetTotals {
    pub total_assets: rust_decimal::Decimal,
    pub total_liabilities: rust_decimal::Decimal,
    pub total_equity: rust_decimal::Decimal,
    pub balance_check: rust_decimal::Decimal,
    pub is_balanced: bool,
}

#[derive(Debug, Serialize)]
pub struct BalanceSheetMetadata {
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub include_zero_balances: bool,
    pub account_count: usize,
}

/// Get balance sheet for today (default: tomorrow to include today's transactions)
pub async fn get_balance_sheet(
    State(database): State<Database>,
    Query(params): Query<BalanceSheetQuery>,
) -> Result<Json<BalanceSheetResponse>, (StatusCode, Json<Value>)> {
    let report_date = chrono::Utc::now().naive_utc().date() + chrono::Duration::days(1);
    get_balance_sheet_for_date(database, report_date, params).await
}

/// Get balance sheet for a specific date
pub async fn get_balance_sheet_by_date(
    State(database): State<Database>,
    Path(date_str): Path<String>,
    Query(params): Query<BalanceSheetQuery>,
) -> Result<Json<BalanceSheetResponse>, (StatusCode, Json<Value>)> {
    let report_date = parse_date(&date_str)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid date format",
                    "message": e,
                    "expected_formats": ["YYYY-MM-DD", "MM/DD/YYYY", "DD/MM/YYYY"]
                })),
            )
        })?;

    get_balance_sheet_for_date(database, report_date, params).await
}

async fn get_balance_sheet_for_date(
    database: Database,
    report_date: NaiveDate,
    params: BalanceSheetQuery,
) -> Result<Json<BalanceSheetResponse>, (StatusCode, Json<Value>)> {
    let report_service = ReportService::new(database.pool().clone());
    
    let balance_sheet = report_service
        .balance_sheet(report_date)
        .await
        .map_err(|e| {
            tracing::error!("Failed to generate balance sheet: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to generate balance sheet",
                    "message": e.to_string()
                })),
            )
        })?;

    let balance_check = balance_sheet.total_assets - balance_sheet.total_liabilities - balance_sheet.total_equity;
    let is_balanced = balance_check.is_zero();

    let account_count = balance_sheet.assets.len() + balance_sheet.liabilities.len() + balance_sheet.equity.len();

    let response = BalanceSheetResponse {
        report_type: "balance_sheet".to_string(),
        report_date: balance_sheet.report_date,
        currency: "EUR".to_string(),
        assets: balance_sheet
            .assets
            .into_iter()
            .map(|a| AccountBalanceResponse {
                name: a.name,
                full_path: a.full_path,
                balance: a.balance,
                level: a.level,
            })
            .collect(),
        liabilities: balance_sheet
            .liabilities
            .into_iter()
            .map(|l| AccountBalanceResponse {
                name: l.name,
                full_path: l.full_path,
                balance: l.balance,
                level: l.level,
            })
            .collect(),
        equity: balance_sheet
            .equity
            .into_iter()
            .map(|e| AccountBalanceResponse {
                name: e.name,
                full_path: e.full_path,
                balance: e.balance,
                level: e.level,
            })
            .collect(),
        totals: BalanceSheetTotals {
            total_assets: balance_sheet.total_assets,
            total_liabilities: balance_sheet.total_liabilities,
            total_equity: balance_sheet.total_equity,
            balance_check,
            is_balanced,
        },
        metadata: BalanceSheetMetadata {
            generated_at: chrono::Utc::now(),
            include_zero_balances: params.include_zero,
            account_count,
        },
    };

    Ok(Json(response))
}

/// Parse date string into NaiveDate
fn parse_date(s: &str) -> Result<NaiveDate, String> {
    // Try various date formats
    if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Ok(date);
    }
    if let Ok(date) = NaiveDate::parse_from_str(s, "%m/%d/%Y") {
        return Ok(date);
    }
    if let Ok(date) = NaiveDate::parse_from_str(s, "%d/%m/%Y") {
        return Ok(date);
    }

    Err(format!(
        "Invalid date format: {}. Expected formats: YYYY-MM-DD, MM/DD/YYYY, DD/MM/YYYY",
        s
    ))
}
