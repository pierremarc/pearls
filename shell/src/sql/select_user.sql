SELECT 
    project, username, task, start_time, end_time , SUM(end_time - start_time)

FROM 
    command_do 
WHERE
    username = :user
    AND start_time > :since
GROUP BY 
    project, task;