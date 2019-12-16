CREATE TABLE IF NOT EXISTS cal(
    id          INTEGER PRIMARY KEY ASC, 
    uuid        TEXT UNIQUE, 
    creation_time  INTEGER, 
    content    TEXT 
    );