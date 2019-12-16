SELECT 
    id, uuid, creation_time, content
FROM 
    cal 
WHERE
    uuid = :uuid;