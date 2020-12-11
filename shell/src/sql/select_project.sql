SELECT project,
  username,
  task,
  start_time,
  end_time,
  end_time - start_time
FROM command_do
WHERE project = :project;