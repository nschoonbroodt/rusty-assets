use assets_core::database::Database;
use assets_core::models::account::Account;
use assets_core::models::transaction::Transaction;
use assets_core::services::{AccountService, ReportService};
use chrono::{Datelike, NaiveDate};
use tauri_plugin_mcp::PluginConfig;

// Tauri commands that expose our backend services to the frontend

#[tauri::command]
async fn get_accounts() -> Result<Vec<Account>, String> {
    let db = Database::from_env().await.map_err(|e| e.to_string())?;
    let service = AccountService::new(db.pool().clone());
    service.get_all_accounts().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_account_by_id(id: String) -> Result<Option<Account>, String> {
    let db = Database::from_env().await.map_err(|e| e.to_string())?;
    let service = AccountService::new(db.pool().clone());
    let uuid = uuid::Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    service.get_account(uuid).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_transactions() -> Result<Vec<Transaction>, String> {
    // For now, return empty list since there's no get_all_transactions method
    // In a real app, you'd implement pagination or specific transaction queries
    Ok(vec![])
}

#[tauri::command]
async fn get_balance_sheet() -> Result<String, String> {
    let db = Database::from_env().await.map_err(|e| e.to_string())?;
    let service = ReportService::new(db.pool().clone());
    let today = chrono::Utc::now().naive_utc().date();
    let _report = service
        .balance_sheet(today)
        .await
        .map_err(|e| e.to_string())?;
    // Return placeholder data since BalanceSheetData doesn't implement Serialize
    Ok("{}".to_string())
}

#[tauri::command]
async fn get_income_statement() -> Result<String, String> {
    let db = Database::from_env().await.map_err(|e| e.to_string())?;
    let service = ReportService::new(db.pool().clone());
    let today = chrono::Utc::now().naive_utc().date();
    let start_of_year = NaiveDate::from_ymd_opt(today.year(), 1, 1).unwrap();
    let _report = service
        .income_statement(start_of_year, today)
        .await
        .map_err(|e| e.to_string())?;
    // Return placeholder data for now
    Ok("[]".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_mcp::init_with_config(
            PluginConfig::new("MyApp".to_string()).tcp("127.0.0.1".to_string(), 4000),
        ))
        .invoke_handler(tauri::generate_handler![
            get_accounts,
            get_account_by_id,
            get_transactions,
            get_balance_sheet,
            get_income_statement
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
