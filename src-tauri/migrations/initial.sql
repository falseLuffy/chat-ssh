-- Create servers table
CREATE TABLE IF NOT EXISTS servers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    host TEXT NOT NULL,
    username TEXT NOT NULL,
    password TEXT,
    port INTEGER NOT NULL DEFAULT 22,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create configuration table
CREATE TABLE IF NOT EXISTS config (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key TEXT UNIQUE NOT NULL,
    value TEXT NOT NULL
);

-- Initial configuration placeholders
INSERT OR IGNORE INTO config (key, value) VALUES ('deepseek_api_key', '');
INSERT OR IGNORE INTO config (key, value) VALUES ('deepseek_model', 'deepseek-chat');
INSERT OR IGNORE INTO config (key, value) VALUES ('ai_mode', 'auto');
