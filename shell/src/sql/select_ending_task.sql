SELECT
  d.id,
  d.username,
  d.start_time,
  d.end_time,
  d.project,
  d.task
FROM command_do as d
LEFT JOIN notif as n ON d.id = n.task_id
WHERE
  n.id IS NULL
  AND (
    CASE
    (d.start_time + ((d.end_time - d.start_time) * 0.9)) < (d.end_time - (300000))
    WHEN 1 THEN (d.end_time - (300000))
    WHEN 0 THEN d.start_time + ((d.end_time - d.start_time) * 0.9)
    END
  ) < :now
  AND :now < d.end_time;