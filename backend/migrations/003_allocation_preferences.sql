-- Create allocation preferences table
CREATE TABLE IF NOT EXISTS allocation_preferences (
    id SERIAL PRIMARY KEY,
    category_name VARCHAR(100) NOT NULL,
    target_percentage DECIMAL(5,2) NOT NULL CHECK (target_percentage >= 0 AND target_percentage <= 100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(category_name)
);

-- Create asset class category mapping table
CREATE TABLE IF NOT EXISTS asset_class_categories (
    id SERIAL PRIMARY KEY,
    asset_class_id INTEGER NOT NULL REFERENCES asset_classes(id) ON DELETE CASCADE,
    category_name VARCHAR(100) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(asset_class_id)
);

-- Insert default preferences (categories)
INSERT INTO allocation_preferences (category_name, target_percentage) VALUES
    ('Cash', 30.00),
    ('Mutual Fund', 30.00),
    ('Stock', 25.00),
    ('Gold', 5.00),
    ('Bitcoin', 10.00)
ON CONFLICT (category_name) DO NOTHING;

-- Map existing asset classes to categories
INSERT INTO asset_class_categories (asset_class_id, category_name)
SELECT id, 
    CASE 
        WHEN name IN ('RDN', 'Bank', 'USD') THEN 'Cash'
        WHEN name = 'Mutual Fund' THEN 'Mutual Fund'
        WHEN name = 'Stock' THEN 'Stock'
        WHEN name = 'Gold' THEN 'Gold'
        WHEN name = 'Bitcoin' THEN 'Bitcoin'
        ELSE 'Other'
    END as category_name
FROM asset_classes
ON CONFLICT (asset_class_id) DO NOTHING;
