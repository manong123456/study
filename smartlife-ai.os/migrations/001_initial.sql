CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT DEFAULT 'user',
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS o_notify (
    custom_id TEXT PRIMARY KEY,
    id TEXT,
    name TEXT,
    icon TEXT,
    state INTEGER DEFAULT 0,
    type INTEGER DEFAULT 0,
    class INTEGER DEFAULT 0,
    message TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS o_shares (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    anonymous INTEGER DEFAULT 0,
    created INTEGER DEFAULT (unixepoch()),
    updated INTEGER DEFAULT (unixepoch())
);

CREATE TABLE IF NOT EXISTS o_connections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT,
    password TEXT,
    host TEXT NOT NULL,
    port TEXT DEFAULT '445',
    status TEXT DEFAULT 'disconnected',
    directories TEXT,
    mount_point TEXT,
    created INTEGER DEFAULT (unixepoch()),
    updated INTEGER DEFAULT (unixepoch())
);

CREATE TABLE IF NOT EXISTS peer_drives (
    id TEXT PRIMARY KEY,
    user_agent TEXT,
    display_name TEXT,
    device_name TEXT,
    model TEXT,
    ip TEXT,
    os TEXT,
    browser TEXT,
    created INTEGER DEFAULT (unixepoch()),
    updated INTEGER DEFAULT (unixepoch())
);
