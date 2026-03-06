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
        .route("/api/allocation-preferences", get(get_allocation_preferences))
        .route("/api/allocation-preferences", post(update_allocation_preferences))
        .route("/api/asset-class-categories", get(get_asset_class_categories))
        .route("/api/asset-class-categories", post(update_asset_class_category))
        .route("/api/rebalancing/calculate", post(calculate_rebalancing))
        .route("/api/telegram/settings", get(get_telegram_settings))
        .route("/api/telegram/settings", post(update_telegram_settings))
        .route("/api/telegram/send", post(send_telegram_report))
        .layer(CorsLayer::permissive())
        .with_state(state.clone());
    
    // Start scheduler for auto-send
    let scheduler_state = state.clone();
    tokio::spawn(async move {
        start_telegram_scheduler(scheduler_state).await;
    });
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();
    
    tracing::info!("Starting server at http://0.0.0.0:8080");
    
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

async fn start_telegram_scheduler(state: Arc<AppState>) {
    use tokio_cron_scheduler::{JobScheduler, Job};
    use chrono::Datelike;
    
    let sched = JobScheduler::new().await.unwrap();
    
    // Run every day at 23:00 and check if it's the last day of the month
    let job = Job::new_async("0 0 23 * * *", move |_uuid, _l| {
        let state = state.clone();
        Box::pin(async move {
            // Check if today is the last day of the month
            let now = chrono::Local::now();
            let tomorrow = now + chrono::Duration::days(1);
            
            // If tomorrow is a different month, today is the last day
            if now.month() != tomorrow.month() {
                tracing::info!("Running scheduled Telegram report (last day of month)");
                
                // Get settings
                let settings_result = sqlx::query_as::<_, models::TelegramSettings>(
                    "SELECT id, bot_token, chat_id, is_enabled, auto_send_enabled, last_sent_at FROM telegram_settings LIMIT 1"
                )
                .fetch_one(&state.db)
                .await;
                
                if let Ok(settings) = settings_result {
                    if settings.auto_send_enabled && settings.is_enabled {
                        if let Some(chat_id) = settings.chat_id {
                            let payload = models::SendTelegramRequest { chat_id };
                            let _ = send_telegram_report(
                                axum::extract::State(state.clone()),
                                Json(payload)
                            ).await;
                        }
                    }
                }
            }
        })
    }).unwrap();
    
    sched.add(job).await.unwrap();
    sched.start().await.unwrap();
    
    // Keep scheduler running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}
