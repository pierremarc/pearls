INSERT INTO
    intent (username, project, amount)
VALUES
    (:username, :project, :amount) 
ON CONFLICT(username, project) 
DO UPDATE
    SET amount = :amount;