-- Update USD asset class to use real-time USD/IDR conversion
UPDATE asset_classes SET
    asset_type = 'USD',
    unit = 'USD',
    price_api_symbol = 'USD'
WHERE name = 'USD';
