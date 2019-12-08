SELECT 
    id, username, start_time, end_time, project, task
FROM 
    command_do 
WHERE
    username = :user
    AND end_time > :now;