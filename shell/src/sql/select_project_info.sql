SELECT 
    id, name, username, start_time, duration
FROM 
    project 
WHERE
    name = :project;