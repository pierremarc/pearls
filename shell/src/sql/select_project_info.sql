SELECT id,
    name,
    username,
    start_time,
    end_time,
    provision,
    completed
FROM project
WHERE name = :project;