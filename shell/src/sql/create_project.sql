CREATE TABLE IF NOT EXISTS project(
    id          INTEGER PRIMARY KEY ASC, 
    name        TEXT UNIQUE, 
    username    TEXT, 
    start_time  INTEGER, 
    duration    INTEGER 
    );