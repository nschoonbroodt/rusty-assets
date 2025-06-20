use tauri::Manager;
use assets_core::services::{AccountService, TransactionService, ReportService};
use assets_core::models::account::Account;
use assets_core::models::transaction::Transaction;
use assets_core::error::Result;

// Tauri commands that expose our backend services to the frontend

#[tauri::command]
async fn get_accounts() -> Result<Vec<Account>, String> {
    let service = AccountService::new().await.map_err(|e| e.to_string())?;
    service.get_all_accounts().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_account_by_id(id: String) -> Result<Option<Account>, String> {
    let service = AccountService::new().await.map_err(|e| e.to_string())?;
    let uuid = uuid::Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    service.get_account_by_id(uuid).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_transactions() -> Result<Vec<Transaction>, String> {
    let service = TransactionService::new().await.map_err(|e| e.to_string())?;
    service.get_all_transactions().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_balance_sheet() -> Result<serde_json::Value, String> {
    let service = ReportService::new().await.map_err(|e| e.to_string())?;
    let report = service.generate_balance_sheet().await.map_err(|e| e.to_string())?;
    serde_json::to_value(report).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_income_statement() -> Result<serde_json::Value, String> {
    let service = ReportService::new().await.map_err(|e| e.to_string())?;
    let report = service.generate_income_statement().await.map_err(|e| e.to_string())?;
    serde_json::to_value(report).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
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