use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AssetClass {
    pub id: i32,
    pub name: String,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateAssetClass {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AssetSnapshot {
    pub id: i32,
    pub snapshot_month: i32,
    pub snapshot_year: i32,
    pub asset_class_id: i32,
    pub amount: f64,
}

#[derive(Debug, Deserialize)]
pub struct CreateSnapshot {
    pub snapshot_month: i32,
    pub snapshot_year: i32,
    pub asset_class_id: i32,
    pub amount: f64,
}

#[derive(Debug, Deserialize)]
pub struct BulkCreateSnapshot {
    pub snapshot_month: i32,
    pub snapshot_year: i32,
    pub assets: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Dividend {
    pub id: i32,
    pub stock_name: String,
    #[serde(serialize_with = "serialize_decimal_as_f64")]
    pub amount: Decimal,
    pub received_month: i32,
    pub received_year: i32,
}

fn serialize_decimal_as_f64<S>(decimal: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use rust_decimal::prelude::ToPrimitive;
    serializer.serialize_f64(decimal.to_f64().unwrap_or(0.0))
}

#[derive(Debug, Deserialize)]
pub struct CreateDividend {
    pub stock_name: String,
    pub amount: f64,
    pub received_month: i32,
    pub received_year: i32,
}
