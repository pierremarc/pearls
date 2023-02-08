PRAGMA user_version = 6;

CREATE UNIQUE INDEX IF NOT EXISTS intent_index ON intent (username, project);