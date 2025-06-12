mod handlers;
mod routes;

use anyhow::Result;
use axum::Router;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "assets_api=debug,tower_http=debug".into()),
        )
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Create the database connection
    let database = assets_core::Database::from_env().await?;

    info!("Connected to database successfully");

    // Build the application with routes
    let app = create_app(database).await?;

    // Start the server
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    info!("🚀 Starting Assets API server on {}", addr);

    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn create_app(database: assets_core::Database) -> Result<Router> {
    let app = Router::new()
        .nest("/api/v1", routes::create_routes())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(database);

    Ok(app)
}
