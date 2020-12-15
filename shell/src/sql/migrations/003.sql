PRAGMA user_version = 3;
CREATE TABLE IF NOT EXISTS note(
    id INTEGER PRIMARY KEY ASC,
    project TEXT,
    username TEXT,
    created_at INTEGER,
    content INTEGER
);