use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PriceCache {
    pub id: i32,
    pub symbol: String,
    pub price_idr: rust_decimal::Decimal,
    pub last_updated: chrono::NaiveDateTime,
    pub source: String,
}

#[derive(Debug, Deserialize)]
struct CoinGeckoResponse {
    bitcoin: CoinGeckoPrice,
}

#[derive(Debug, Deserialize)]
struct CoinGeckoPrice {
    idr: f64,
}

/// Fetch Bitcoin price from CoinGecko API (free, no API key needed)
async fn fetch_bitcoin_price() -> Result<f64, Box<dyn std::error::Error>> {
    let url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=idr";
    
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("User-Agent", "WealthPortfolioTracker/1.0")
        .send()
        .await?;
    
    let data: CoinGeckoResponse = response.json().await?;
    Ok(data.bitcoin.idr)
}

/// Fetch Gold price per gram in IDR from harga-emas.org
/// This website provides real Indonesian gold prices
async fn fetch_gold_price() -> Result<f64, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    // Try to scrape from harga-emas.org (Indonesian gold price website)
    let harga_emas_url = "https://harga-emas.org";
    
    eprintln!("Fetching gold price from harga-emas.org...");
    
    match client
        .get(harga_emas_url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(response) => {
            if let Ok(html) = response.text().await {
                // Parse HTML to extract gold price per gram in IDR
                // Looking for pattern like "Gram (gr)152,53(-1,93)2.613.709,99(-32.438,71)"
                // We want the IDR price: 2.613.709,99
                
                // Try to find the price pattern
                if let Some(start) = html.find("Gram (gr)") {
                    let substring = &html[start..];

                    // Page format: "Gram (gr)152,43(-1,9)2.613.015,55(-29.580,17)"
                    // We want IDR price (has at least 2 dot groups AND value between 1M-10M)
                    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
                    let re = RE.get_or_init(|| regex::Regex::new(r"(\d{1,3}(?:\.\d{3})+,\d{2})").unwrap());

                    for captures in re.captures_iter(substring) {
                        if let Some(price_str) = captures.get(1) {
                            let price_clean = price_str.as_str()
                                .replace(".", "")
                                .replace(",", ".");
                            if let Ok(price) = price_clean.parse::<f64>() {
                                // IDR gold per gram should be between 1M and 10M
                                if price >= 1_000_000.0 && price <= 10_000_000.0 {
                                    eprintln!("Gold price from harga-emas.org: Rp {:.2}/gram", price);
                                    return Ok(price);
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch from harga-emas.org: {:?}", e);
        }
    }
    
    // Fallback: Use Yahoo Finance if harga-emas.org fails
    eprintln!("harga-emas.org failed, trying Yahoo Finance...");
    
    let yahoo_url = "https://query1.finance.yahoo.com/v8/finance/chart/GC=F";
    
    match client
        .get(yahoo_url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(response) => {
            #[derive(Deserialize)]
            struct YahooResponse {
                chart: YahooChart,
            }
            
            #[derive(Deserialize)]
            struct YahooChart {
                result: Vec<YahooResult>,
            }
            
            #[derive(Deserialize)]
            struct YahooResult {
                meta: YahooMeta,
            }
            
            #[derive(Deserialize)]
            struct YahooMeta {
                #[serde(rename = "regularMarketPrice")]
                regular_market_price: Option<f64>,
            }
            
            if let Ok(data) = response.json::<YahooResponse>().await {
                if let Some(result) = data.chart.result.first() {
                    if let Some(price_usd_per_oz) = result.meta.regular_market_price {
                        // Get USD/IDR exchange rate
                        let exchange_url = "https://api.exchangerate-api.com/v4/latest/USD";
                        
                        #[derive(Deserialize)]
                        struct ExchangeRateResponse {
                            rates: HashMap<String, f64>,
                        }
                        
                        if let Ok(ex_response) = client.get(exchange_url).send().await {
                            if let Ok(ex_data) = ex_response.json::<ExchangeRateResponse>().await {
                                let usd_to_idr = ex_data.rates.get("IDR").copied().unwrap_or(16000.0);
                                let price_usd_per_gram = price_usd_per_oz / 31.1035;
                                let price_idr_per_gram = price_usd_per_gram * usd_to_idr;
                                let retail_price = price_idr_per_gram * 1.10;
                                
                                eprintln!("Gold price from Yahoo Finance: Rp {:.2}/gram", retail_price);
                                return Ok(retail_price);
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Yahoo Finance also failed: {:?}", e);
        }
    }
    
    // Final fallback: Use fixed calculation
    eprintln!("All sources failed, using fallback calculation");
    
    let usd_idr_url = "https://api.exchangerate-api.com/v4/latest/USD";
    
    #[derive(Deserialize)]
    struct ExchangeRateResponse {
        rates: HashMap<String, f64>,
    }
    
    let exchange_response = client.get(usd_idr_url).send().await?;
    let exchange_data: ExchangeRateResponse = exchange_response.json().await?;
    let usd_to_idr = exchange_data.rates.get("IDR").copied().unwrap_or(16000.0);
    
    // Use approximate current gold spot price ($2900 per troy oz as of April 2026)
    let gold_usd_per_oz = 2900.0;
    let gold_usd_per_gram = gold_usd_per_oz / 31.1035;
    let gold_idr_per_gram = gold_usd_per_gram * usd_to_idr;
    
    // Add 10% markup for Indonesian retail market
    let retail_price = gold_idr_per_gram * 1.10;
    
    eprintln!("Gold price calculated (fallback): Rp {:.2}/gram", retail_price);
    
    Ok(retail_price)
}

/// Update price cache in database
pub async fn update_price_cache(
    db: &PgPool,
    symbol: &str,
    price_idr: f64,
    source: &str,
) -> Result<(), sqlx::Error> {
    let price_decimal = rust_decimal::Decimal::from_f64_retain(price_idr).unwrap_or(rust_decimal::Decimal::ZERO);
    
    sqlx::query(
        r#"
        INSERT INTO price_cache (symbol, price_idr, source, last_updated)
        VALUES ($1, $2, $3, CURRENT_TIMESTAMP)
        ON CONFLICT (symbol)
        DO UPDATE SET 
            price_idr = $2,
            source = $3,
            last_updated = CURRENT_TIMESTAMP
        "#
    )
    .bind(symbol)
    .bind(price_decimal)
    .bind(source)
    .execute(db)
    .await?;
    
    Ok(())
}

/// Get cached price from database
pub async fn get_cached_price(
    db: &PgPool,
    symbol: &str,
    max_age_minutes: i64,
) -> Result<Option<f64>, sqlx::Error> {
    let result = sqlx::query_as::<_, PriceCache>(
        r#"
        SELECT id, symbol, price_idr, last_updated, source
        FROM price_cache
        WHERE symbol = $1
        AND last_updated > CURRENT_TIMESTAMP - INTERVAL '1 minute' * $2
        "#
    )
    .bind(symbol)
    .bind(max_age_minutes)
    .fetch_optional(db)
    .await?;
    
    Ok(result.map(|p| p.price_idr.to_string().parse::<f64>().unwrap_or(0.0)))
}

/// Fetch USD/IDR exchange rate
async fn fetch_usd_rate() -> Result<f64, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    #[derive(Deserialize)]
    struct ExchangeRateResponse {
        rates: HashMap<String, f64>,
    }

    let url = "https://api.exchangerate-api.com/v4/latest/USD";
    let response = client
        .get(url)
        .header("User-Agent", "WealthPortfolioTracker/1.0")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    let data: ExchangeRateResponse = response.json().await?;
    let rate = data.rates.get("IDR").copied().ok_or("IDR rate not found")?;
    eprintln!("USD/IDR rate: {:.2}", rate);
    Ok(rate)
}

/// Fetch and cache USD/IDR rate
pub async fn get_usd_rate(db: &PgPool) -> Result<f64, Box<dyn std::error::Error>> {
    // Cache for 60 minutes - exchange rates don't change that fast
    if let Some(cached) = get_cached_price(db, "USD", 60).await? {
        return Ok(cached);
    }
    let rate = fetch_usd_rate().await?;
    update_price_cache(db, "USD", rate, "exchangerate-api.com").await?;
    Ok(rate)
}

/// Fetch and cache Bitcoin price
pub async fn get_bitcoin_price(db: &PgPool) -> Result<f64, Box<dyn std::error::Error>> {
    // Check cache first (5 minutes for crypto)
    if let Some(cached_price) = get_cached_price(db, "bitcoin", 5).await? {
        return Ok(cached_price);
    }
    
    // Fetch fresh price
    let price = fetch_bitcoin_price().await?;
    update_price_cache(db, "bitcoin", price, "coingecko").await?;
    
    Ok(price)
}

/// Fetch and cache Gold price per gram
pub async fn get_gold_price(db: &PgPool) -> Result<f64, Box<dyn std::error::Error>> {
    // Check cache first (30 minutes for gold - Indonesian market updates)
    if let Some(cached_price) = get_cached_price(db, "XAU", 30).await? {
        return Ok(cached_price);
    }
    
    // Fetch fresh price
    let price = fetch_gold_price().await?;
    update_price_cache(db, "XAU", price, "harga-emas.org").await?;
    
    Ok(price)
}

/// Get price for any asset by symbol
pub async fn get_price_by_symbol(
    db: &PgPool,
    symbol: &str,
) -> Result<f64, Box<dyn std::error::Error>> {
    match symbol {
        "bitcoin" => get_bitcoin_price(db).await,
        "XAU" => get_gold_price(db).await,
        "USD" => get_usd_rate(db).await,
        _ => Err("Unsupported symbol".into()),
    }
}

/// Convert unit amount to IDR
pub async fn convert_to_idr(
    db: &PgPool,
    unit_amount: f64,
    asset_type: &str,
    price_api_symbol: Option<&str>,
) -> Result<f64, Box<dyn std::error::Error>> {
    match asset_type {
        "CURRENCY" => Ok(unit_amount), // Already in IDR
        "USD" | "COMMODITY" | "CRYPTO" => {
            if let Some(symbol) = price_api_symbol {
                let price = get_price_by_symbol(db, symbol).await?;
                Ok(unit_amount * price)
            } else {
                Err("Missing price_api_symbol for non-currency asset".into())
            }
        }
        _ => Ok(unit_amount), // Default to direct value
    }
}
