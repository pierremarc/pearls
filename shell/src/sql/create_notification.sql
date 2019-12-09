CREATE TABLE IF NOT EXISTS notif(
    id          INTEGER PRIMARY KEY ASC, 
    task_id     INTEGER, 
    end_time    INTEGER, 
    FOREIGN KEY(task_id) REFERENCES command_do(id)
    );