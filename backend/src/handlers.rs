use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use rust_decimal::prelude::*;
use std::sync::Arc;
use crate::{AppState, models::*};

macro_rules! db_err {
    ($msg:expr) => {
        |e| {
            tracing::error!("{}: {:?}", $msg, e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    };
}

pub async fn get_asset_classes(
    State(state): State<Arc<AppState>>
) -> Result<Json<Vec<AssetClass>>, StatusCode> {
    let classes = sqlx::query_as::<_, AssetClass>(
        "SELECT id, name, is_active, asset_type, unit, price_api_symbol FROM asset_classes WHERE is_active = true ORDER BY name"
    )
    .fetch_all(&state.db)
    .await
    .map_err(db_err!("Error fetching asset classes"))?;

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
    .map_err(db_err!("Error creating asset class"))?;

    Ok((StatusCode::CREATED, Json(class)))
}
pub async fn delete_asset_class(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<i32>
) -> Result<StatusCode, StatusCode> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM asset_snapshots WHERE asset_class_id = $1 AND amount > 0"
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(db_err!("Error checking snapshots"))?;

    if count.0 > 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    sqlx::query("DELETE FROM asset_snapshots WHERE asset_class_id = $1 AND amount = 0")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(db_err!("Error deleting zero snapshots"))?;

    sqlx::query("DELETE FROM asset_classes WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(db_err!("Error deleting asset class"))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_snapshots(
    State(state): State<Arc<AppState>>
) -> Result<Json<Vec<AssetSnapshot>>, StatusCode> {
    let snapshots = sqlx::query_as::<_, AssetSnapshot>(
        "SELECT id, snapshot_month, snapshot_year, asset_class_id, amount FROM asset_snapshots ORDER BY snapshot_year DESC, snapshot_month DESC, asset_class_id"
    )
    .fetch_all(&state.db)
    .await
    .map_err(db_err!("Error fetching snapshots"))?;

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
    use crate::price_service;
    
    let mut tx = state.db.begin().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    for (asset_name, amount) in payload.assets.iter() {
        let asset_class = sqlx::query_as::<_, AssetClass>(
            "SELECT id, name, is_active, asset_type, unit, price_api_symbol FROM asset_classes WHERE name = $1"
        )
        .bind(asset_name)
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
        
        // Get unit_amount if provided and non-zero
        let unit_amount = payload.unit_amounts.as_ref()
            .and_then(|units| units.get(asset_name).copied())
            .filter(|&u| u != 0.0); // treat exactly 0 as "not provided" — keep existing IDR value
        
        // Only convert if unit_amount is explicitly provided and > 0
        let final_amount = if let Some(unit_amt) = unit_amount {
            let asset_type = asset_class.asset_type.as_deref().unwrap_or("CURRENCY");
            if asset_type != "CURRENCY" {
                price_service::convert_to_idr(
                    &state.db,
                    unit_amt,
                    asset_type,
                    asset_class.price_api_symbol.as_deref(),
                )
                .await
                .unwrap_or(*amount)
            } else {
                *amount
            }
        } else {
            *amount // unit empty → keep the IDR amount as-is
        };
        
        sqlx::query(
            "INSERT INTO asset_snapshots (snapshot_month, snapshot_year, asset_class_id, amount, unit_amount) 
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (snapshot_year, snapshot_month, asset_class_id) 
             DO UPDATE SET amount = $4, unit_amount = $5"
        )
        .bind(payload.snapshot_month)
        .bind(payload.snapshot_year)
        .bind(asset_class.id)
        .bind(final_amount)
        .bind(unit_amount)
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
    .map_err(db_err!("Error fetching dividends"))?;

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
    }

    let latest_snapshots = sqlx::query_as::<_, SnapshotRow>(
        r#"
        SELECT ac.name, s.amount
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
    .map_err(db_err!("Error fetching dashboard data"))?;

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
        .map_err(db_err!("Error fetching total dividends"))?;

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
        unit_amount: Option<rust_decimal::Decimal>,
    }

    let history = sqlx::query_as::<_, HistoryRow>(
        r#"
        SELECT s.snapshot_month, s.snapshot_year, ac.name, s.amount, s.unit_amount
        FROM asset_snapshots s
        INNER JOIN asset_classes ac ON s.asset_class_id = ac.id
        ORDER BY s.snapshot_year ASC, s.snapshot_month ASC, ac.name ASC
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(db_err!("Error fetching history"))?;

    let mut grouped: std::collections::HashMap<String, (std::collections::HashMap<String, f64>, std::collections::HashMap<String, f64>)> = std::collections::HashMap::new();

    for row in history {
        let key = format!("{}-{:02}", row.snapshot_year, row.snapshot_month);
        let amount_f64 = row.amount.to_f64().unwrap_or(0.0);
        let unit_f64 = row.unit_amount.and_then(|u| u.to_f64()).unwrap_or(0.0);
        let entry = grouped.entry(key).or_insert_with(|| (std::collections::HashMap::new(), std::collections::HashMap::new()));
        entry.0.insert(row.name.clone(), amount_f64);
        entry.1.insert(row.name, unit_f64);
    }

    let mut result: Vec<_> = grouped.into_iter().map(|(date, (assets, unit_amounts))| {
        let total: f64 = assets.values().sum();
        serde_json::json!({
            "date": date,
            "assets": assets,
            "unit_amounts": unit_amounts,
            "total": total
        })
    }).collect();

    result.sort_by(|a, b| a["date"].as_str().cmp(&b["date"].as_str()));

    Ok(Json(serde_json::json!(result)))
}

// Category Management Handlers
pub async fn get_categories(
    State(state): State<Arc<AppState>>
) -> Result<Json<Vec<AllocationPreference>>, StatusCode> {
    let categories = sqlx::query_as::<_, AllocationPreference>(
        "SELECT id, category_name, target_percentage FROM allocation_preferences ORDER BY category_name"
    )
    .fetch_all(&state.db)
    .await
    .map_err(db_err!("Error fetching categories"))?;

    Ok(Json(categories))
}

pub async fn create_category(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateCategory>
) -> Result<(StatusCode, Json<AllocationPreference>), StatusCode> {
    let category = sqlx::query_as::<_, AllocationPreference>(
        "INSERT INTO allocation_preferences (category_name, target_percentage) VALUES ($1, 0) RETURNING id, category_name, target_percentage"
    )
    .bind(&payload.category_name)
    .fetch_one(&state.db)
    .await
    .map_err(db_err!("Error creating category"))?;

    Ok((StatusCode::CREATED, Json(category)))
}

pub async fn delete_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>
) -> Result<StatusCode, StatusCode> {
    let category = sqlx::query_as::<_, AllocationPreference>(
        "SELECT id, category_name, target_percentage FROM allocation_preferences WHERE id = $1"
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(db_err!("Error fetching category"))?;

    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM asset_class_categories WHERE category_name = $1"
    )
    .bind(&category.category_name)
    .fetch_one(&state.db)
    .await
    .map_err(db_err!("Error checking category usage"))?;

    if count.0 > 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    sqlx::query("DELETE FROM allocation_preferences WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(db_err!("Error deleting category"))?;

    Ok(StatusCode::NO_CONTENT)
}

// Allocation Preferences Handlers
pub async fn get_allocation_preferences(
    State(state): State<Arc<AppState>>
) -> Result<Json<Vec<AllocationPreference>>, StatusCode> {
    let preferences = sqlx::query_as::<_, AllocationPreference>(
        "SELECT id, category_name, target_percentage FROM allocation_preferences ORDER BY target_percentage DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(db_err!("Error fetching allocation preferences"))?;

    Ok(Json(preferences))
}

pub async fn update_allocation_preferences(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Vec<UpdateAllocationPreference>>
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    // Validate total percentage equals 100
    let total: f64 = payload.iter().map(|p| p.target_percentage).sum();
    const TOLERANCE: f64 = 0.01;
    if (total - 100.0).abs() > TOLERANCE {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let mut tx = state.db.begin().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    for pref in payload.iter() {
        let decimal_percentage = Decimal::from_f64(pref.target_percentage).unwrap_or(Decimal::ZERO);
        sqlx::query(
            "INSERT INTO allocation_preferences (category_name, target_percentage) 
             VALUES ($1, $2)
             ON CONFLICT (category_name) 
             DO UPDATE SET target_percentage = $2, updated_at = CURRENT_TIMESTAMP"
        )
        .bind(&pref.category_name)
        .bind(decimal_percentage)
        .execute(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    
    tx.commit().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok((StatusCode::OK, Json(serde_json::json!({"success": true}))))
}

pub async fn get_asset_class_categories(
    State(state): State<Arc<AppState>>
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    #[derive(sqlx::FromRow)]
    struct AssetClassWithCategory {
        asset_class_id: i32,
        asset_class_name: String,
        category_name: Option<String>,
    }

    let mappings = sqlx::query_as::<_, AssetClassWithCategory>(
        r#"
        SELECT 
            ac.id as asset_class_id,
            ac.name as asset_class_name,
            acc.category_name
        FROM asset_classes ac
        LEFT JOIN asset_class_categories acc ON ac.id = acc.asset_class_id
        WHERE ac.is_active = true
        ORDER BY ac.name
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(db_err!("Error fetching asset class categories"))?;

    let result: Vec<_> = mappings.iter().map(|m| {
        serde_json::json!({
            "asset_class_id": m.asset_class_id,
            "asset_class_name": m.asset_class_name,
            "category_name": m.category_name.as_deref().unwrap_or("Other")
        })
    }).collect();

    Ok(Json(result))
}

pub async fn update_asset_class_category(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateAssetClassCategory>
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    sqlx::query(
        "INSERT INTO asset_class_categories (asset_class_id, category_name) 
         VALUES ($1, $2)
         ON CONFLICT (asset_class_id) 
         DO UPDATE SET category_name = $2"
    )
    .bind(payload.asset_class_id)
    .bind(&payload.category_name)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok((StatusCode::OK, Json(serde_json::json!({"success": true}))))
}

pub async fn calculate_rebalancing(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RebalancingInput>
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get current assets with their categories
    #[derive(sqlx::FromRow)]
    struct CurrentAssetWithCategory {
        name: String,
        amount: rust_decimal::Decimal,
        category_name: Option<String>,
    }
    
    let current_assets = sqlx::query_as::<_, CurrentAssetWithCategory>(
        r#"
        SELECT 
            ac.name, 
            s.amount,
            acc.category_name
        FROM asset_snapshots s
        INNER JOIN asset_classes ac ON s.asset_class_id = ac.id
        LEFT JOIN asset_class_categories acc ON ac.id = acc.asset_class_id
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
    .map_err(db_err!("Error fetching current assets for rebalancing"))?;
    
    // Group by category
    let mut grouped_assets: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    let mut category_breakdown: std::collections::HashMap<String, Vec<(String, f64)>> = std::collections::HashMap::new();
    
    for asset in current_assets.iter() {
        let amount = asset.amount.to_f64().unwrap_or(0.0);
        let category = asset.category_name.as_deref().unwrap_or("Other").to_string();

        *grouped_assets.entry(category.clone()).or_insert(0.0) += amount;
        category_breakdown.entry(category).or_insert_with(Vec::new).push((asset.name.clone(), amount));
    }
    
    let current_total: f64 = current_assets.iter()
        .map(|a| a.amount.to_f64().unwrap_or(0.0))
        .sum();
    
    let new_total = current_total + payload.additional_amount;
    
    // Get allocation preferences
    let preferences = sqlx::query_as::<_, AllocationPreference>(
        "SELECT id, category_name, target_percentage FROM allocation_preferences ORDER BY target_percentage DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(db_err!("Error fetching preferences for rebalancing"))?;
    
    // Calculate target amounts and allocations
    let mut recommendations = Vec::new();
    let mut total_positive_difference = 0.0;
    let mut temp_recommendations = Vec::new();
    
    // First pass: calculate differences and total positive difference
    for pref in preferences.iter() {
        let target_percentage = pref.target_percentage.to_f64().unwrap_or(0.0);
        let target_amount = new_total * (target_percentage / 100.0);
        let current_amount = grouped_assets.get(&pref.category_name).copied().unwrap_or(0.0);
        let difference = target_amount - current_amount;
        
        if difference > 0.0 {
            total_positive_difference += difference;
        }
        
        temp_recommendations.push((pref.category_name.clone(), target_percentage, current_amount, target_amount, difference));
    }
    
    // Second pass: allocate additional amount proportionally
    for (category_name, target_percentage, current_amount, target_amount, difference) in temp_recommendations {
        let allocation_from_new = if difference > 0.0 && total_positive_difference > 0.0 {
            // Allocate proportionally based on how much this category needs
            let proportion = difference / total_positive_difference;
            (payload.additional_amount * proportion).min(difference)
        } else {
            0.0
        };
        
        // Get breakdown for this category
        let breakdown = category_breakdown.get(&category_name).map(|items| {
            items.iter().map(|(name, amount)| {
                serde_json::json!({
                    "name": name,
                    "amount": amount
                })
            }).collect::<Vec<_>>()
        });
        
        recommendations.push(serde_json::json!({
            "asset_class": category_name,
            "target_percentage": target_percentage,
            "current_amount": current_amount,
            "target_amount": target_amount,
            "difference": difference,
            "suggested_allocation": allocation_from_new,
            "breakdown": breakdown
        }));
    }
    
    Ok(Json(serde_json::json!({
        "current_total": current_total,
        "additional_amount": payload.additional_amount,
        "new_total": new_total,
        "recommendations": recommendations
    })))
}

// Telegram Settings Handlers
pub async fn get_telegram_settings(
    State(state): State<Arc<AppState>>
) -> Result<Json<TelegramSettings>, StatusCode> {
    let settings = sqlx::query_as::<_, TelegramSettings>(
        "SELECT id, bot_token, chat_id, is_enabled, auto_send_enabled, last_sent_at FROM telegram_settings LIMIT 1"
    )
    .fetch_one(&state.db)
    .await
    .map_err(db_err!("Error fetching telegram settings"))?;

    Ok(Json(settings))
}

pub async fn update_telegram_settings(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateTelegramSettings>
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    sqlx::query(
        "UPDATE telegram_settings SET bot_token = $1, chat_id = $2, is_enabled = $3, auto_send_enabled = $4, updated_at = CURRENT_TIMESTAMP"
    )
    .bind(&payload.bot_token)
    .bind(&payload.chat_id)
    .bind(payload.is_enabled)
    .bind(payload.auto_send_enabled)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok((StatusCode::OK, Json(serde_json::json!({"success": true}))))
}

pub async fn send_telegram_report(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SendTelegramRequest>
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    // Get telegram settings
    let settings = sqlx::query_as::<_, TelegramSettings>(
        "SELECT id, bot_token, chat_id, is_enabled, auto_send_enabled, last_sent_at FROM telegram_settings LIMIT 1"
    )
    .fetch_one(&state.db)
    .await
    .map_err(db_err!("Error fetching telegram settings for report"))?;

    if settings.bot_token.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Get current dashboard data
    let dashboard_data = get_dashboard_data(State(state.clone())).await?;
    
    // Get previous month data for comparison
    let previous_data = get_previous_month_data(&state.db).await.unwrap_or(PreviousMonthData {
        total: 0.0,
        per_asset: std::collections::HashMap::new(),
    });
    
    // Format text message
    let message = format_telegram_message(&dashboard_data.0, &previous_data);
    
    // Send text message to Telegram
    let client = reqwest::Client::new();
    let url = format!("https://api.telegram.org/bot{}/sendMessage", settings.bot_token);
    
    let response = client
        .post(&url)
        .json(&serde_json::json!({
            "chat_id": payload.chat_id,
            "text": message,
            "parse_mode": "HTML"
        }))
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if !response.status().is_success() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    // Update last sent timestamp
    sqlx::query("UPDATE telegram_settings SET last_sent_at = CURRENT_TIMESTAMP")
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok((StatusCode::OK, Json(serde_json::json!({"success": true, "message": "Report sent successfully"}))))
}

struct PreviousMonthData {
    total: f64,
    per_asset: std::collections::HashMap<String, f64>,
}

async fn get_previous_month_data(db: &sqlx::PgPool) -> Result<PreviousMonthData, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct PrevAssetRow {
        name: String,
        amount: rust_decimal::Decimal,
    }
    
    let rows = sqlx::query_as::<_, PrevAssetRow>(
        r#"
        SELECT ac.name, s.amount
        FROM asset_snapshots s
        INNER JOIN asset_classes ac ON s.asset_class_id = ac.id
        WHERE (s.snapshot_year, s.snapshot_month) = (
            SELECT snapshot_year, snapshot_month 
            FROM asset_snapshots 
            WHERE (snapshot_year, snapshot_month) < (
                SELECT snapshot_year, snapshot_month 
                FROM asset_snapshots 
                ORDER BY snapshot_year DESC, snapshot_month DESC 
                LIMIT 1
            )
            ORDER BY snapshot_year DESC, snapshot_month DESC 
            LIMIT 1
        )
        "#
    )
    .fetch_all(db)
    .await?;
    
    let mut per_asset = std::collections::HashMap::new();
    let mut total = 0.0;
    
    for row in rows {
        let amount = row.amount.to_f64().unwrap_or(0.0);
        total += amount;
        per_asset.insert(row.name, amount);
    }
    
    Ok(PreviousMonthData { total, per_asset })
}

fn format_rupiah(amount: f64) -> String {
    let amount_str = format!("{:.0}", amount);
    let mut result = String::new();
    let chars: Vec<char> = amount_str.chars().collect();
    
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push('.');
        }
        result.push(*c);
    }
    
    result
}

pub async fn send_pdf_report(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SendPdfRequest>
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let settings = sqlx::query_as::<_, TelegramSettings>(
        "SELECT id, bot_token, chat_id, is_enabled, auto_send_enabled, last_sent_at FROM telegram_settings LIMIT 1"
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if settings.bot_token.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    use base64::Engine;
    let pdf_bytes = base64::engine::general_purpose::STANDARD
        .decode(&payload.pdf_base64)
        .map_err(|e| {
            eprintln!("Base64 decode error: {:?}", e);
            StatusCode::BAD_REQUEST
        })?;

    let client = reqwest::Client::new();
    let url = format!("https://api.telegram.org/bot{}/sendDocument", settings.bot_token);

    let part = reqwest::multipart::Part::bytes(pdf_bytes)
        .file_name(payload.filename.clone())
        .mime_str("application/pdf")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let form = reqwest::multipart::Form::new()
        .text("chat_id", payload.chat_id.clone())
        .text("caption", "📊 Wealth Portfolio Report (PDF)")
        .part("document", part);

    let response = client
        .post(&url)
        .multipart(form)
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        eprintln!("Telegram error: {}", body);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok((StatusCode::OK, Json(serde_json::json!({"success": true}))))
}

fn format_telegram_message(data: &serde_json::Value, previous: &PreviousMonthData) -> String {
    use chrono_tz::Asia::Jakarta;
    
    let total = data["total"].as_f64().unwrap_or(0.0);
    let total_dividends = data["total_dividends"].as_f64().unwrap_or(0.0);
    let empty_vec = vec![];
    let allocations = data["allocations"].as_array().unwrap_or(&empty_vec);
    
    let mut message = String::from("📊 <b>Financial Report</b>\n\n");
    message.push_str(&format!("💰 <b>Total Assets:</b> Rp {}\n", format_rupiah(total)));
    
    // Add comparison with previous month
    if previous.total > 0.0 {
        let difference = total - previous.total;
        let percentage_change = (difference / previous.total) * 100.0;
        
        if difference > 0.0 {
            message.push_str(&format!("   📈 <i>+Rp {} ({:+.2}%) from last month</i>\n", format_rupiah(difference), percentage_change));
        } else if difference < 0.0 {
            message.push_str(&format!("   📉 <i>Rp {} ({:.2}%) from last month</i>\n", format_rupiah(difference), percentage_change));
        } else {
            message.push_str("   ➡️ <i>No change from last month</i>\n");
        }
    }
    
    message.push_str(&format!("\n💵 <b>Total Dividends:</b> Rp {}\n\n", format_rupiah(total_dividends)));
    message.push_str("<b>Asset Allocation:</b>\n");
    
    for allocation in allocations {
        let name = allocation["name"].as_str().unwrap_or("");
        let amount = allocation["amount"].as_f64().unwrap_or(0.0);
        let percentage = allocation["percentage"].as_f64().unwrap_or(0.0);
        message.push_str(&format!("• {}: Rp {} ({:.2}%)\n", name, format_rupiah(amount), percentage));
        
        // Per-asset comparison with previous month
        if let Some(&prev_amount) = previous.per_asset.get(name) {
            if prev_amount > 0.0 {
                let diff = amount - prev_amount;
                let pct_change = (diff / prev_amount) * 100.0;
                if diff > 0.0 {
                    message.push_str(&format!("   📈 <i>+Rp {} ({:+.2}%)</i>\n", format_rupiah(diff), pct_change));
                } else if diff < 0.0 {
                    message.push_str(&format!("   📉 <i>-Rp {} ({:.2}%)</i>\n", format_rupiah(diff.abs()), pct_change));
                } else {
                    message.push_str("   ➡️ <i>No change</i>\n");
                }
            }
        } else if !previous.per_asset.is_empty() {
            message.push_str("   🆕 <i>New asset</i>\n");
        }
    }
    
    // Get current time in Jakarta timezone (GMT+7)
    let jakarta_time = chrono::Utc::now().with_timezone(&Jakarta);
    message.push_str(&format!("\n📅 Generated: {} WIB", jakarta_time.format("%d %B %Y, %H:%M")));
    
    message
}

// Price Service Handlers
pub async fn get_current_prices(
    State(state): State<Arc<AppState>>
) -> Result<Json<serde_json::Value>, StatusCode> {
    use crate::price_service;
    
    let mut prices = serde_json::Map::new();
    
    // Fetch Bitcoin price
    match price_service::get_bitcoin_price(&state.db).await {
        Ok(price) => {
            prices.insert("bitcoin".to_string(), serde_json::json!({
                "symbol": "BTC",
                "price_idr": price,
                "unit": "BTC"
            }));
        }
        Err(e) => {
            eprintln!("Error fetching Bitcoin price: {:?}", e);
        }
    }
    
    // Fetch Gold price
    match price_service::get_gold_price(&state.db).await {
        Ok(price) => {
            prices.insert("gold".to_string(), serde_json::json!({
                "symbol": "XAU",
                "price_idr": price,
                "unit": "gram"
            }));
        }
        Err(e) => {
            eprintln!("Error fetching Gold price: {:?}", e);
        }
    }

    // Fetch USD/IDR rate
    match price_service::get_usd_rate(&state.db).await {
        Ok(rate) => {
            prices.insert("usd".to_string(), serde_json::json!({
                "symbol": "USD",
                "price_idr": rate,
                "unit": "USD"
            }));
        }
        Err(e) => {
            eprintln!("Error fetching USD rate: {:?}", e);
        }
    }
    
    Ok(Json(serde_json::json!(prices)))
}

pub async fn convert_unit_to_idr(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ConvertRequest>
) -> Result<Json<serde_json::Value>, StatusCode> {
    use crate::price_service;
    
    // Get asset class info
    let asset_class = sqlx::query_as::<_, AssetClass>(
        "SELECT id, name, is_active, asset_type, unit, price_api_symbol FROM asset_classes WHERE id = $1"
    )
    .bind(payload.asset_class_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Error fetching asset class: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let asset_type = asset_class.asset_type.as_deref().unwrap_or("CURRENCY");
    let price_api_symbol = asset_class.price_api_symbol.as_deref();
    
    let idr_amount = price_service::convert_to_idr(
        &state.db,
        payload.unit_amount,
        asset_type,
        price_api_symbol,
    )
    .await
    .map_err(|e| {
        eprintln!("Error converting to IDR: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(serde_json::json!({
        "unit_amount": payload.unit_amount,
        "idr_amount": idr_amount,
        "asset_type": asset_type,
        "unit": asset_class.unit.unwrap_or("IDR".to_string())
    })))
}
