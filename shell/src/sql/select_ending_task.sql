SELECT 
    d.id, d.username, d.end_time
FROM 
    command_do as d
    LEFT JOIN notif as n 
        ON d.id = n.task_id 
WHERE
    (n.id IS NULL OR n.end_time <> d.end_time)
    AND (d.start_time + ((d.end_time - d.start_time) * 0.9)) < :now  ;
    AND :now < d.end_time 