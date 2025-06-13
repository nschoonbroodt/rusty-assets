use anyhow::Result;
use assets_core::{Database, ReportService};
use chrono::NaiveDate;
use clap::Args;

mod balance_sheet;

/// Generate balance sheet report
pub async fn generate_balance_sheet(params: BalanceSheetParams) -> Result<()> {
    let db = Database::from_env().await?;
    let report_service = ReportService::new(db.pool().clone());
    let report_date = params
        .date
        .unwrap_or_else(|| chrono::Utc::now().naive_utc().date() + chrono::Duration::days(1));
    let balance_sheet_data = report_service.balance_sheet(report_date).await?;

    match params.format.as_str() {
        "json" => balance_sheet::print_balance_sheet_json(&balance_sheet_data)?, // TODO: should use the params
        "csv" => balance_sheet::print_balance_sheet_csv(&balance_sheet_data)?, // TODO: should use the params
        _ => balance_sheet::print_balance_sheet_table(&balance_sheet_data, &params)?,
    }

    Ok(())
}

/// Generate income statement report
pub async fn generate_income_statement(params: IncomeStatementParams) -> Result<()> {
    todo!(
        "Generate income statement from {:?} to {:?}",
        params.start_date,
        params.end_date
    );
}

/// Generate cash flow statement
pub async fn generate_cash_flow_statement(params: CashFlowParams) -> Result<()> {
    todo!(
        "Generate cash flow statement from {:?} to {:?}",
        params.start_date,
        params.end_date
    );
}

/// Generate trial balance report
pub async fn generate_trial_balance(params: TrialBalanceParams) -> Result<()> {
    todo!("Generate trial balance for date: {:?}", params.date);
}

/// Generate account ledger report
pub async fn generate_account_ledger(params: AccountLedgerParams) -> Result<()> {
    todo!(
        "Generate account ledger for account: {} from {:?} to {:?}",
        params.account_path,
        params.start_date,
        params.end_date
    );
}

/// Generate net worth report over time
pub async fn generate_net_worth_report(params: NetWorthParams) -> Result<()> {
    todo!(
        "Generate net worth report from {:?} to {:?}",
        params.start_date,
        params.end_date
    );
}

/// Generate budget vs actual report
pub async fn generate_budget_report(params: BudgetReportParams) -> Result<()> {
    todo!(
        "Generate budget vs actual report for period: {:?} to {:?}",
        params.start_date,
        params.end_date
    );
}

/// Generate expense analysis report
pub async fn generate_expense_analysis(params: ExpenseAnalysisParams) -> Result<()> {
    todo!(
        "Generate expense analysis from {:?} to {:?}, category: {:?}",
        params.start_date,
        params.end_date,
        params.category_filter
    );
}

/// Generate investment performance report
pub async fn generate_investment_performance(params: InvestmentPerformanceParams) -> Result<()> {
    todo!(
        "Generate investment performance report from {:?} to {:?}",
        params.start_date,
        params.end_date
    );
}

/// Generate tax report
pub async fn generate_tax_report(params: TaxReportParams) -> Result<()> {
    todo!("Generate tax report for year: {}", params.year);
}

/// Parameters for balance sheet report
#[derive(Args)]
pub struct BalanceSheetParams {
    /// Date for the balance sheet (default: tomorrow)
    #[arg(long, value_parser = parse_date)]
    pub date: Option<NaiveDate>,

    /// Include zero balances
    #[arg(long)]
    pub include_zero: bool,

    /// Output format: table, csv, json
    #[arg(long, default_value = "table")]
    pub format: String,
}

/// Parameters for income statement report
#[derive(Args)]
pub struct IncomeStatementParams {
    /// Start date for the period
    #[arg(long, value_parser = parse_date)]
    pub start_date: Option<NaiveDate>,

    /// End date for the period (default: today)
    #[arg(long, value_parser = parse_date)]
    pub end_date: Option<NaiveDate>,

    /// Output format: table, csv, json
    #[arg(long, default_value = "table")]
    pub format: String,
}

/// Parameters for cash flow statement
#[derive(Args)]
pub struct CashFlowParams {
    /// Start date for the period
    #[arg(long, value_parser = parse_date)]
    pub start_date: Option<NaiveDate>,

    /// End date for the period (default: today)
    #[arg(long, value_parser = parse_date)]
    pub end_date: Option<NaiveDate>,

    /// Output format: table, csv, json
    #[arg(long, default_value = "table")]
    pub format: String,
}

/// Parameters for trial balance report
#[derive(Args)]
pub struct TrialBalanceParams {
    /// Date for the trial balance (default: today)
    #[arg(long, value_parser = parse_date)]
    pub date: Option<NaiveDate>,

    /// Include zero balances
    #[arg(long)]
    pub include_zero: bool,

    /// Output format: table, csv, json
    #[arg(long, default_value = "table")]
    pub format: String,
}

/// Parameters for account ledger report
#[derive(Args)]
pub struct AccountLedgerParams {
    /// Account path (e.g., "Assets:Current Assets:Main Checking")
    pub account_path: String,

    /// Start date for the period
    #[arg(long, value_parser = parse_date)]
    pub start_date: Option<NaiveDate>,

    /// End date for the period (default: today)
    #[arg(long, value_parser = parse_date)]
    pub end_date: Option<NaiveDate>,

    /// Show running balance
    #[arg(long)]
    pub show_balance: bool,

    /// Output format: table, csv, json
    #[arg(long, default_value = "table")]
    pub format: String,
}

/// Parameters for net worth report
#[derive(Args)]
pub struct NetWorthParams {
    /// Start date for the period
    #[arg(long, value_parser = parse_date)]
    pub start_date: Option<NaiveDate>,

    /// End date for the period (default: today)
    #[arg(long, value_parser = parse_date)]
    pub end_date: Option<NaiveDate>,

    /// Frequency: daily, weekly, monthly, yearly
    #[arg(long, default_value = "monthly")]
    pub frequency: String,

    /// Output format: table, csv, json
    #[arg(long, default_value = "table")]
    pub format: String,
}

/// Parameters for budget report
#[derive(Args)]
pub struct BudgetReportParams {
    /// Start date for the period
    #[arg(long, value_parser = parse_date)]
    pub start_date: Option<NaiveDate>,

    /// End date for the period (default: today)
    #[arg(long, value_parser = parse_date)]
    pub end_date: Option<NaiveDate>,

    /// Budget name/version to compare against
    #[arg(long)]
    pub budget_name: Option<String>,

    /// Show only variances above threshold
    #[arg(long)]
    pub variance_threshold: Option<f64>,

    /// Output format: table, csv, json
    #[arg(long, default_value = "table")]
    pub format: String,
}

/// Parameters for expense analysis report
#[derive(Args)]
pub struct ExpenseAnalysisParams {
    /// Start date for the period
    #[arg(long, value_parser = parse_date)]
    pub start_date: Option<NaiveDate>,

    /// End date for the period (default: today)
    #[arg(long, value_parser = parse_date)]
    pub end_date: Option<NaiveDate>,

    /// Filter by category pattern
    #[arg(long)]
    pub category_filter: Option<String>,

    /// Group by: category, month, week
    #[arg(long, default_value = "category")]
    pub group_by: String,

    /// Output format: table, csv, json
    #[arg(long, default_value = "table")]
    pub format: String,
}

/// Parameters for investment performance report
#[derive(Args)]
pub struct InvestmentPerformanceParams {
    /// Start date for the period
    #[arg(long, value_parser = parse_date)]
    pub start_date: Option<NaiveDate>,

    /// End date for the period (default: today)
    #[arg(long, value_parser = parse_date)]
    pub end_date: Option<NaiveDate>,

    /// Filter by symbol
    #[arg(long)]
    pub symbol: Option<String>,

    /// Include dividends
    #[arg(long)]
    pub include_dividends: bool,

    /// Output format: table, csv, json
    #[arg(long, default_value = "table")]
    pub format: String,
}

/// Parameters for tax report
#[derive(Args)]
pub struct TaxReportParams {
    /// Tax year
    pub year: i32,

    /// Tax jurisdiction (US, CA, etc.)
    #[arg(long, default_value = "US")]
    pub jurisdiction: String,

    /// Include only transactions above threshold
    #[arg(long)]
    pub threshold: Option<f64>,

    /// Output format: table, csv, json
    #[arg(long, default_value = "table")]
    pub format: String,
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
