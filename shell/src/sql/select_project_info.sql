SELECT
    id,
    name,
    username,
    start_time,
    end_time,
    provision
FROM
    project
WHERE
    name = :project;