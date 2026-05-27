use axum::{
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use std::sync::Arc;

mod handlers;
mod models;
mod price_service;

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
        .max_connections(10)
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
        .route("/api/asset-classes", get(get_asset_classes).post(create_asset_class))
        .route("/api/asset-classes/:id", delete(delete_asset_class))
        .route("/api/snapshots", get(get_snapshots).post(create_snapshot))
        .route("/api/snapshots/bulk", post(bulk_create_snapshot))
        .route("/api/snapshots/:id", put(update_snapshot).delete(delete_snapshot))
        .route("/api/dividends", get(get_dividends).post(create_dividend))
        .route("/api/dividends/:id", put(update_dividend).delete(delete_dividend))
        .route("/api/dashboard", get(get_dashboard_data))
        .route("/api/history", get(get_history))
        .route("/api/categories", get(get_categories).post(create_category))
        .route("/api/categories/:id", delete(delete_category))
        .route("/api/allocation-preferences", get(get_allocation_preferences).post(update_allocation_preferences))
        .route("/api/asset-class-categories", get(get_asset_class_categories).post(update_asset_class_category))
        .route("/api/rebalancing/calculate", post(calculate_rebalancing))
        .route("/api/telegram/settings", get(get_telegram_settings).post(update_telegram_settings))
        .route("/api/telegram/send", post(send_telegram_report))
        .route("/api/telegram/send-pdf", post(send_pdf_report))
        .route("/api/prices/current", get(get_current_prices))
        .route("/api/prices/convert", post(convert_unit_to_idr))
        .layer(CorsLayer::permissive())
        .layer(axum::extract::DefaultBodyLimit::max(50 * 1024 * 1024))
        .with_state(state.clone());

    tokio::spawn(async move {
        start_telegram_scheduler(state).await;
    });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();

    tracing::info!("Server listening on http://0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn start_telegram_scheduler(state: Arc<AppState>) {
    use chrono::Datelike;
    use tokio_cron_scheduler::{Job, JobScheduler};

    let Ok(sched) = JobScheduler::new().await else {
        tracing::error!("Failed to create job scheduler");
        return;
    };

    let job = Job::new_async("0 0 23 * * *", move |_uuid, _l| {
        let state = state.clone();
        Box::pin(async move {
            let now = chrono::Local::now();
            let tomorrow = now + chrono::Duration::days(1);

            if now.month() != tomorrow.month() {
                tracing::info!("Running scheduled Telegram report (last day of month)");

                let Ok(settings) = sqlx::query_as::<_, models::TelegramSettings>(
                    "SELECT id, bot_token, chat_id, is_enabled, auto_send_enabled, last_sent_at FROM telegram_settings LIMIT 1"
                )
                .fetch_one(&state.db)
                .await else {
                    tracing::error!("Failed to fetch Telegram settings for scheduler");
                    return;
                };

                if settings.auto_send_enabled && settings.is_enabled {
                    if let Some(chat_id) = settings.chat_id {
                        let payload = models::SendTelegramRequest { chat_id };
                        if let Err(e) = send_telegram_report(
                            axum::extract::State(state.clone()),
                            Json(payload),
                        ).await {
                            tracing::error!("Scheduled Telegram send failed: {:?}", e);
                        }
                    }
                }
            }
        })
    }).unwrap();

    sched.add(job).await.unwrap();
    sched.start().await.unwrap();

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}
