use axum::{
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use std::sync::Arc;

mod models;
mod handlers;

use handlers::*;

#[derive(Clone)]
pub struct AppState {
    db: sqlx::PgPool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/asset_tracker".to_string());
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");
    
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    let state = Arc::new(AppState { db: pool });
    
    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/asset-classes", get(get_asset_classes))
        .route("/api/asset-classes", post(create_asset_class))
        .route("/api/snapshots", get(get_snapshots))
        .route("/api/snapshots", post(create_snapshot))
        .route("/api/snapshots/bulk", post(bulk_create_snapshot))
        .route("/api/snapshots/:id", put(update_snapshot))
        .route("/api/snapshots/:id", delete(delete_snapshot))
        .route("/api/dividends", get(get_dividends))
        .route("/api/dividends", post(create_dividend))
        .route("/api/dividends/:id", put(update_dividend))
        .route("/api/dividends/:id", delete(delete_dividend))
        .route("/api/dashboard", get(get_dashboard_data))
        .route("/api/history", get(get_history))
        .layer(CorsLayer::permissive())
        .with_state(state);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();
    
    tracing::info!("Starting server at http://0.0.0.0:8080");
    
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}
