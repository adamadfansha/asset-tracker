-- Drop old dividends table and recreate with new schema
DROP TABLE IF EXISTS dividends;

CREATE TABLE dividends (
    id SERIAL PRIMARY KEY,
    stock_name VARCHAR(255) NOT NULL,
    amount NUMERIC(20, 2) NOT NULL,
    received_month INTEGER NOT NULL,
    received_year INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
