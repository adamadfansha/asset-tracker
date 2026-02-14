CREATE TABLE IF NOT EXISTS asset_classes (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS asset_snapshots (
    id SERIAL PRIMARY KEY,
    snapshot_month INTEGER NOT NULL,
    snapshot_year INTEGER NOT NULL,
    asset_class_id INTEGER NOT NULL REFERENCES asset_classes(id),
    amount NUMERIC(20, 2) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(snapshot_year, snapshot_month, asset_class_id)
);

CREATE TABLE IF NOT EXISTS dividends (
    id SERIAL PRIMARY KEY,
    stock_name VARCHAR(255) NOT NULL,
    amount NUMERIC(20, 2) NOT NULL,
    received_date DATE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO asset_classes (name) VALUES 
    ('Stock'),
    ('Mutual Fund'),
    ('USD'),
    ('Gold'),
    ('RDN'),
    ('Bank'),
    ('Bitcoin')
ON CONFLICT (name) DO NOTHING;
