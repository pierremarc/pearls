PRAGMA user_version = 1;

ALTER TABLE
    project
ADD
    COLUMN end_time INTEGER;

ALTER TABLE
    project
ADD
    COLUMN provision INTEGER;