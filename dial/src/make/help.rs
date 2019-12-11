pub fn help() -> Option<(String, String)> {
    Some((
        "
        !ping
            check if the bot's still alive
        !new <project-name> <hours>
            register a new project
        !do <project-name> <task-name> <duration>
            start a new task
        !stop
            stop your current task
        !more <duration>
            add some time to your current task. the new end will NOW + <duration>
        !ls
            list current tasks
        !project <project-name>
            give stat for a given project
        !since <date or duration>
            a summary of your tasks since date
        "
        .into(),
        "
        <h4>!ping</h4>
            check if the bot's still alive
        <h4>!new <em>project-name</em> <em>hours</em></h4>
            register a new project
        <h4>!do <em>project-name</em> <em>task-name</em> <em>duration</em></h4>
            <p>start a new task</p>
            <p>you'll be notified of its ending</p>
        <h4>!stop</h4>
            stop your current task
        <h4>!more <em>duration</em></h4>
            add some time to your current task. the new end will <em>now</em> + <em>duration</em>
        <h4>!ls</h4>
            list current tasks
        <h4>!project <em>project-name</em></h4>
            give stat for a given project
        <h4>!since <em>date-or-duration</em></h4>
            a summary of your tasks since date
        "
        .into(),
    ))
}
