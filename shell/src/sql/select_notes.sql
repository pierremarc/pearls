SELECT id,
    username,
    project,
    created_at,
    content
FROM note
WHERE project = :project;