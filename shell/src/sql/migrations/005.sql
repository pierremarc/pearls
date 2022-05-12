PRAGMA user_version = 5;

CREATE TABLE IF NOT EXISTS avail(
    id INTEGER PRIMARY KEY ASC,
    username TEXT,
    start_time INTEGER,
    end_time INTEGER,
    weekly INTEGER
);

CREATE TABLE IF NOT EXISTS intent(
    id INTEGER PRIMARY KEY ASC,
    username TEXT,
    project TEXT,
    amount INTEGER
);