CREATE TABLE IF NOT EXISTS command_do(
    id          INTEGER PRIMARY KEY ASC, 
    username    TEXT, 
    start_time  INTEGER, 
    end_time    INTEGER, 
    project     TEXT, 
    task        TEXT
    );