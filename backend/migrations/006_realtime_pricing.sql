-- Add asset type and unit to asset_classes
ALTER TABLE asset_classes 
ADD COLUMN asset_type VARCHAR(50) DEFAULT 'CURRENCY',
ADD COLUMN unit VARCHAR(50) DEFAULT 'IDR',
ADD COLUMN price_api_symbol VARCHAR(100);

-- Add unit_amount to asset_snapshots to store original unit value
ALTER TABLE asset_snapshots
ADD COLUMN unit_amount NUMERIC(20, 8) DEFAULT 0;

-- Create price_cache table for real-time prices
CREATE TABLE IF NOT EXISTS price_cache (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR(100) NOT NULL UNIQUE,
    price_idr NUMERIC(20, 2) NOT NULL,
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    source VARCHAR(100) NOT NULL
);

-- Update existing asset classes with proper types
UPDATE asset_classes SET 
    asset_type = 'COMMODITY',
    unit = 'gram',
    price_api_symbol = 'XAU'
WHERE name = 'Gold';

UPDATE asset_classes SET 
    asset_type = 'CRYPTO',
    unit = 'BTC',
    price_api_symbol = 'bitcoin'
WHERE name = 'Bitcoin';

UPDATE asset_classes SET 
    asset_type = 'CURRENCY',
    unit = 'IDR'
WHERE name IN ('Stock', 'Mutual Fund', 'USD', 'RDN', 'Bank');

-- Create index for faster price lookups
CREATE INDEX idx_price_cache_symbol ON price_cache(symbol);
CREATE INDEX idx_price_cache_updated ON price_cache(last_updated);

COMMENT ON COLUMN asset_classes.asset_type IS 'Type: CURRENCY, COMMODITY, CRYPTO, STOCK';
COMMENT ON COLUMN asset_classes.unit IS 'Unit of measurement: IDR, gram, BTC, shares';
COMMENT ON COLUMN asset_classes.price_api_symbol IS 'Symbol for price API lookup';
COMMENT ON COLUMN asset_snapshots.unit_amount IS 'Original amount in units (e.g., 5 grams, 0.05 BTC)';
