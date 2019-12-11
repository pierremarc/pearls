SELECT 
    id, username, start_time, end_time, project, task, max(end_time)
FROM 
    command_do 
WHERE
    username = :user;