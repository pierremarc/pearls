SELECT
    id,
    username,
    start_time,
    end_time,
    weekly
FROM
    avail
WHERE
    end_time > :now;