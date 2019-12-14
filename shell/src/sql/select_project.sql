SELECT 
    project, username, task, start_time, end_time , SUM(end_time - start_time)
FROM 
    command_do 
WHERE
    project = :project
GROUP BY 
    project, username, task;