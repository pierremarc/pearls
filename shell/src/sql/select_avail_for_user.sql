SELECT
    id,
    username,
    start_time,
    end_time,
    weekly
FROM
    avail
WHERE
    username = :user
    AND end_time > :now;