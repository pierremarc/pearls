SELECT
    i.id,
    i.username,
    i.project,
    i.amount,
    p.start_time,
    p.completed
FROM
    intent AS i
    LEFT JOIN project AS p ON i.project = p.name
WHERE
    p.completed IS NULL
    OR p.completed < :now;