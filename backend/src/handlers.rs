use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;
use rust_decimal::prelude::*;
use crate::{AppState, models::*};

pub async fn get_asset_classes(
    State(state): State<Arc<AppState>>
) -> Result<Json<Vec<AssetClass>>, StatusCode> {
    let classes = sqlx::query_as::<_, AssetClass>(
        "SELECT id, name, is_active FROM asset_classes WHERE is_active = true ORDER BY name"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(classes))
}

pub async fn create_asset_class(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateAssetClass>
) -> Result<(StatusCode, Json<AssetClass>), StatusCode> {
    let class = sqlx::query_as::<_, AssetClass>(
        "INSERT INTO asset_classes (name) VALUES ($1) RETURNING id, name, is_active"
    )
    .bind(&payload.name)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok((StatusCode::CREATED, Json(class)))
}

pub async fn get_snapshots(
    State(state): State<Arc<AppState>>
) -> Result<Json<Vec<AssetSnapshot>>, StatusCode> {
    let snapshots = sqlx::query_as::<_, AssetSnapshot>(
        "SELECT id, snapshot_month, snapshot_year, asset_class_id, amount FROM asset_snapshots ORDER BY snapshot_year DESC, snapshot_month DESC, asset_class_id"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(snapshots))
}

pub async fn create_snapshot(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateSnapshot>
) -> Result<(StatusCode, Json<AssetSnapshot>), StatusCode> {
    let snapshot = sqlx::query_as::<_, AssetSnapshot>(
        "INSERT INTO asset_snapshots (snapshot_month, snapshot_year, asset_class_id, amount) VALUES ($1, $2, $3, $4) RETURNING id, snapshot_month, snapshot_year, asset_class_id, amount"
    )
    .bind(payload.snapshot_month)
    .bind(payload.snapshot_year)
    .bind(payload.asset_class_id)
    .bind(payload.amount)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok((StatusCode::CREATED, Json(snapshot)))
}

pub async fn bulk_create_snapshot(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<BulkCreateSnapshot>
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let mut tx = state.db.begin().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    for (asset_name, amount) in payload.assets.iter() {
        let asset_class = sqlx::query_as::<_, AssetClass>(
            "SELECT id, name, is_active FROM asset_classes WHERE name = $1"
        )
        .bind(asset_name)
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
        
        sqlx::query(
            "INSERT INTO asset_snapshots (snapshot_month, snapshot_year, asset_class_id, amount) 
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (snapshot_year, snapshot_month, asset_class_id) 
             DO UPDATE SET amount = $4"
        )
        .bind(payload.snapshot_month)
        .bind(payload.snapshot_year)
        .bind(asset_class.id)
        .bind(amount)
        .execute(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    
    tx.commit().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok((StatusCode::CREATED, Json(serde_json::json!({"success": true}))))
}

pub async fn update_snapshot(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<CreateSnapshot>
) -> Result<Json<AssetSnapshot>, StatusCode> {
    let snapshot = sqlx::query_as::<_, AssetSnapshot>(
        "UPDATE asset_snapshots SET snapshot_month = $1, snapshot_year = $2, asset_class_id = $3, amount = $4 WHERE id = $5 RETURNING id, snapshot_month, snapshot_year, asset_class_id, amount"
    )
    .bind(payload.snapshot_month)
    .bind(payload.snapshot_year)
    .bind(payload.asset_class_id)
    .bind(payload.amount)
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(snapshot))
}

pub async fn delete_snapshot(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM asset_snapshots WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_dividends(
    State(state): State<Arc<AppState>>
) -> Result<Json<Vec<Dividend>>, StatusCode> {
    let dividends = sqlx::query_as::<_, Dividend>(
        "SELECT id, stock_name, amount, received_month, received_year FROM dividends ORDER BY received_year DESC, received_month DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(dividends))
}

pub async fn create_dividend(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateDividend>
) -> Result<(StatusCode, Json<Dividend>), StatusCode> {
    let dividend = sqlx::query_as::<_, Dividend>(
        "INSERT INTO dividends (stock_name, amount, received_month, received_year) VALUES ($1, $2, $3, $4) RETURNING id, stock_name, amount, received_month, received_year"
    )
    .bind(&payload.stock_name)
    .bind(payload.amount)
    .bind(payload.received_month)
    .bind(payload.received_year)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok((StatusCode::CREATED, Json(dividend)))
}

pub async fn update_dividend(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<CreateDividend>
) -> Result<Json<Dividend>, StatusCode> {
    let dividend = sqlx::query_as::<_, Dividend>(
        "UPDATE dividends SET stock_name = $1, amount = $2, received_month = $3, received_year = $4 WHERE id = $5 RETURNING id, stock_name, amount, received_month, received_year"
    )
    .bind(&payload.stock_name)
    .bind(payload.amount)
    .bind(payload.received_month)
    .bind(payload.received_year)
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(dividend))
}

pub async fn delete_dividend(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM dividends WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_dashboard_data(
    State(state): State<Arc<AppState>>
) -> Result<Json<serde_json::Value>, StatusCode> {
    #[derive(sqlx::FromRow)]
    struct SnapshotRow {
        name: String,
        amount: rust_decimal::Decimal,
        snapshot_month: i32,
        snapshot_year: i32,
    }
    
    let latest_snapshots = sqlx::query_as::<_, SnapshotRow>(
        r#"
        SELECT ac.name, s.amount, s.snapshot_month, s.snapshot_year
        FROM asset_snapshots s
        INNER JOIN asset_classes ac ON s.asset_class_id = ac.id
        WHERE (s.snapshot_year, s.snapshot_month) = (
            SELECT snapshot_year, snapshot_month 
            FROM asset_snapshots 
            ORDER BY snapshot_year DESC, snapshot_month DESC 
            LIMIT 1
        )
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Error fetching dashboard data: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let total: f64 = latest_snapshots.iter().map(|s| s.amount.to_f64().unwrap_or(0.0)).sum();
    
    let allocations: Vec<_> = latest_snapshots.iter().map(|s| {
        let amount_f64 = s.amount.to_f64().unwrap_or(0.0);
        serde_json::json!({
            "name": s.name,
            "amount": amount_f64,
            "percentage": if total > 0.0 { (amount_f64 / total) * 100.0 } else { 0.0 }
        })
    }).collect();
    
    let total_dividends: rust_decimal::Decimal = sqlx::query_scalar("SELECT COALESCE(SUM(amount), 0) FROM dividends")
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            eprintln!("Error fetching dividends: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(Json(serde_json::json!({
        "total": total,
        "allocations": allocations,
        "total_dividends": total_dividends.to_f64().unwrap_or(0.0)
    })))
}

pub async fn get_history(
    State(state): State<Arc<AppState>>
) -> Result<Json<serde_json::Value>, StatusCode> {
    #[derive(sqlx::FromRow)]
    struct HistoryRow {
        snapshot_month: i32,
        snapshot_year: i32,
        name: String,
        amount: rust_decimal::Decimal,
    }
    
    let history = sqlx::query_as::<_, HistoryRow>(
        r#"
        SELECT s.snapshot_month, s.snapshot_year, ac.name, s.amount
        FROM asset_snapshots s
        INNER JOIN asset_classes ac ON s.asset_class_id = ac.id
        ORDER BY s.snapshot_year ASC, s.snapshot_month ASC, ac.name ASC
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Error fetching history: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let mut grouped: std::collections::HashMap<String, std::collections::HashMap<String, f64>> = std::collections::HashMap::new();
    
    for row in history {
        let key = format!("{}-{:02}", row.snapshot_year, row.snapshot_month);
        let amount_f64 = row.amount.to_f64().unwrap_or(0.0);
        grouped.entry(key).or_insert_with(std::collections::HashMap::new).insert(row.name, amount_f64);
    }
    
    let mut result: Vec<_> = grouped.into_iter().map(|(date, assets)| {
        let total: f64 = assets.values().sum();
        serde_json::json!({
            "date": date,
            "assets": assets,
            "total": total
        })
    }).collect();
    
    result.sort_by(|a, b| a["date"].as_str().cmp(&b["date"].as_str()));
    
    Ok(Json(serde_json::json!(result)))
}
