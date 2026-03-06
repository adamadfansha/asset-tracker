-- Create telegram settings table
CREATE TABLE IF NOT EXISTS telegram_settings (
    id SERIAL PRIMARY KEY,
    bot_token TEXT NOT NULL,
    chat_id TEXT,
    is_enabled BOOLEAN DEFAULT false,
    auto_send_enabled BOOLEAN DEFAULT false,
    last_sent_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert default settings
INSERT INTO telegram_settings (bot_token, is_enabled, auto_send_enabled) 
VALUES ('', false, false);
