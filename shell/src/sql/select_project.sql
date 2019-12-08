SELECT 
    project, username, task, SUM(end_time - start_time)
FROM 
    command_do 
WHERE
    project = :project
GROUP BY 
    project, username, task;