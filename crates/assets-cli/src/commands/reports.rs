use anyhow::Result;
use assets_core::{Database, ReportService, UserService};
use clap::{Args, ValueEnum};
use uuid::Uuid;

use crate::{DateRange, SingleDate};

mod account_ledger;
mod balance_sheet;
mod cash_flow;
mod income_statement;

/// Helper function to get user UUID from username
async fn get_user_id_by_name(username: &str) -> Result<Uuid> {
    let db = Database::from_env().await?;
    let user_service = UserService::new(db.pool().clone());

    match user_service.get_user_by_name(username).await? {
        Some(user) => Ok(user.id),
        None => Err(anyhow::anyhow!("User '{}' not found", username)),
    }
}

/// Output format for reports
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    /// Display as a formatted table (default)
    Table,
    /// Export as JSON
    Json,
    /// Export as CSV
    Csv,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Table
    }
}

/// Generate balance sheet report
pub async fn generate_balance_sheet(params: BalanceSheetParams) -> Result<()> {
    println!("{:?}", params);
    let db = Database::from_env().await?;
    let report_service = ReportService::new(db.pool().clone());
    let report_date = params.date.get_date();
    let balance_sheet_data = report_service.balance_sheet(report_date).await?;

    match params.format {
        OutputFormat::Json => balance_sheet::print_balance_sheet_json(&balance_sheet_data)?,
        OutputFormat::Csv => balance_sheet::print_balance_sheet_csv(&balance_sheet_data)?,
        OutputFormat::Table => {
            balance_sheet::print_balance_sheet_table(&balance_sheet_data, &params)?
        }
    }

    Ok(())
}

/// Generate income statement report
pub async fn generate_income_statement(params: IncomeStatementParams) -> Result<()> {
    let db = Database::from_env().await?;
    let report_service = ReportService::new(db.pool().clone());

    let (start_date, end_date) = params.date_range.range();

    let user_uuid = get_user_id_by_name(&params.user).await?;

    let income_statement_data = report_service
        .income_statement(start_date, end_date, user_uuid) // user_uuid is already a Uuid
        .await?;
    match params.format {
        OutputFormat::Json => {
            income_statement::print_income_statement_json(&income_statement_data)?;
        }
        OutputFormat::Csv => {
            income_statement::print_income_statement_csv(&income_statement_data)?;
        }
        OutputFormat::Table => {
            income_statement::print_income_statement_table(&income_statement_data)?;
        }
    }

    Ok(())
}

/// Generate cash flow statement
pub async fn generate_cash_flow_statement(params: CashFlowParams) -> Result<()> {
    let db = Database::from_env().await?;
    let report_service = ReportService::new(db.pool().clone());

    // Get user ID from username
    let user_id = get_user_id_by_name(&params.user).await?;

    let (start_date, end_date) = params.date_range.range();

    let cash_flow_data = report_service
        .cash_flow_statement(start_date, end_date, user_id)
        .await?;
    match params.format {
        OutputFormat::Json => {
            cash_flow::print_cash_flow_json(&cash_flow_data, start_date, end_date)?
        }
        OutputFormat::Csv => cash_flow::print_cash_flow_csv(&cash_flow_data)?,
        OutputFormat::Table => {
            cash_flow::print_cash_flow_table(&cash_flow_data, start_date, end_date)?
        }
    }

    Ok(())
}

/// Generate trial balance report
pub async fn generate_trial_balance(params: TrialBalanceParams) -> Result<()> {
    let report_date = params.date.get_date();
    todo!("Generate trial balance for date: {:?}", report_date);
}

/// Generate account ledger report
pub async fn generate_account_ledger(params: AccountLedgerParams) -> Result<()> {
    let db = Database::from_env().await?;
    let report_service = ReportService::new(db.pool().clone());
    // Find account by path
    let account_service = assets_core::AccountService::new(db.pool().clone());
    let account = account_service
        .get_account_by_path(&params.account_path)
        .await
        .map_err(|_| anyhow::anyhow!("Account '{}' not found", params.account_path))?;

    // Set default dates if not provided
    let (start_date, end_date) = params.date_range.range();

    let ledger_data = report_service
        .account_ledger(account.id, start_date, end_date)
        .await?;
    match params.format {
        OutputFormat::Json => {
            account_ledger::print_account_ledger_json(&ledger_data, &account, start_date, end_date)?
        }
        OutputFormat::Csv => {
            account_ledger::print_account_ledger_csv(&ledger_data, &account, start_date, end_date)?
        }
        OutputFormat::Table => account_ledger::print_account_ledger_table(
            &ledger_data,
            &account,
            start_date,
            end_date,
            params.show_balance,
        )?,
    }

    Ok(())
}

/// Generate net worth report over time
pub async fn generate_net_worth_report(params: NetWorthParams) -> Result<()> {
    let (start_date, end_date) = params.date_range.range();
    todo!(
        "Generate net worth report from {:?} to {:?}",
        start_date,
        end_date
    );
}

/// Generate budget vs actual report
pub async fn generate_budget_report(params: BudgetReportParams) -> Result<()> {
    let (start_date, end_date) = params.date_range.range();
    todo!(
        "Generate budget vs actual report for period: {:?} to {:?}",
        start_date,
        end_date
    );
}

/// Generate expense analysis report
pub async fn generate_expense_analysis(params: ExpenseAnalysisParams) -> Result<()> {
    let (start_date, end_date) = params.date_range.range();
    todo!(
        "Generate expense analysis from {:?} to {:?}, category: {:?}",
        start_date,
        end_date,
        params.category_filter
    );
}

/// Generate investment performance report
pub async fn generate_investment_performance(params: InvestmentPerformanceParams) -> Result<()> {
    let (start_date, end_date) = params.date_range.range();
    todo!(
        "Generate investment performance report from {:?} to {:?}",
        start_date,
        end_date
    );
}

/// Parameters for balance sheet report
#[derive(Args, Debug)]
pub struct BalanceSheetParams {
    #[command(flatten)]
    pub date: SingleDate,

    /// Include zero balances
    #[arg(long)]
    pub include_zero: bool,

    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,
}

/// Parameters for income statement report
#[derive(Args)]
pub struct IncomeStatementParams {
    /// Username for the report
    #[arg(long)]
    pub user: String,
    #[command(flatten)]
    pub date_range: DateRange,
    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,
}

/// Parameters for cash flow statement
#[derive(Args)]
pub struct CashFlowParams {
    /// Username for the report
    #[arg(long)]
    pub user: String,
    #[command(flatten)]
    pub date_range: DateRange,

    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,
}

/// Parameters for trial balance report
#[derive(Args, Debug)]
pub struct TrialBalanceParams {
    #[command(flatten)]
    pub date: SingleDate,

    /// Include zero balances
    #[arg(long)]
    pub include_zero: bool,

    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,
}

/// Parameters for account ledger report
#[derive(Args)]
pub struct AccountLedgerParams {
    /// Account path (e.g., "Assets:Current Assets:Main Checking")
    pub account_path: String,
    #[command(flatten)]
    pub date_range: DateRange,
    /// Show running balance
    #[arg(long)]
    pub show_balance: bool,

    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,
}

/// Parameters for net worth report
#[derive(Args)]
pub struct NetWorthParams {
    #[command(flatten)]
    pub date_range: DateRange,
    /// Frequency: daily, weekly, monthly, yearly
    #[arg(long, default_value = "monthly")]
    pub frequency: String,

    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,
}

/// Parameters for budget report
#[derive(Args)]
pub struct BudgetReportParams {
    #[command(flatten)]
    pub date_range: DateRange,

    /// Budget name/version to compare against
    #[arg(long)]
    pub budget_name: Option<String>,
    /// Show only variances above threshold
    #[arg(long)]
    pub variance_threshold: Option<f64>,

    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,
}

/// Parameters for expense analysis report
#[derive(Args)]
pub struct ExpenseAnalysisParams {
    #[command(flatten)]
    pub date_range: DateRange,

    /// Filter by category pattern
    #[arg(long)]
    pub category_filter: Option<String>,
    /// Group by: category, month, week
    #[arg(long, default_value = "category")]
    pub group_by: String,

    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,
}

/// Parameters for investment performance report
#[derive(Args)]
pub struct InvestmentPerformanceParams {
    #[command(flatten)]
    pub date_range: DateRange,

    /// Filter by symbol
    #[arg(long)]
    pub symbol: Option<String>,
    /// Include dividends
    #[arg(long)]
    pub include_dividends: bool,

    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,
}
