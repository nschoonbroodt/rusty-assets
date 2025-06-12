use assets_core::Database;
use axum::{routing::get, Router};

use crate::handlers;

pub fn create_routes() -> Router<Database> {
    Router::new()
        // Health check endpoint
        .route("/health", get(handlers::health::health_check))
        // Report endpoints
        .nest("/reports", create_report_routes())
}

fn create_report_routes() -> Router<Database> {
    Router::new()
        .route("/balance-sheet", get(handlers::reports::get_balance_sheet))
        .route(
            "/balance-sheet/:date",
            get(handlers::reports::get_balance_sheet_by_date),
        )
}
