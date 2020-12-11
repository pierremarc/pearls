pub fn help() -> Option<(String, String)> {
    Some((
        "
        !ping
            check if the bot's still alive
        !new <project-name>
            register a new project
        !deadline <project-name> <end time>
            set a deadline for an existing project
        !provision <project-name> <duration>
            set provisioned time for an existing project
        !complete <date?> 
            set completion date for an existing project, if
            date is not provided, it will take the current time instead.
        !do <project-name> <task-name> <duration>
            start a new task that will last for <duration>
        !done <project-name> <task-name> <duration>
            register a task that has lasted for <duration> from now
        !switch <project-name> <task-name>
            stop your current task and create a new one that has the same end time as the current one
        !stop
            stop your current task
        !more <duration>
            stop your current task and create a new one with same project and task for <duration>
        !ls
            list current tasks
        !digest <project-name>
            give stat for a given project
        !since <date or duration>
            a summary of your tasks since date
        "
        .into(),
        "
        <h4>!ping</h4>
            check if the bot's still alive
        <h4>!new <em>project-name</em></h4>
            register a new project
        <h4>!deadline <em>project-name</em> <em>end-time</em></h4>
            set a deadline for an existing project
        <h4>!provision <em>project-name</em> <em>duration</em></h4>
            set provisioned time for an existing project
        <h4>!complete <em>date?-name</em></h4>
            set completion date for an existing project, if
            date is not provided, it will take the current time instead.
        <h4>!do <em>project-name</em> <em>task-name</em> <em>duration</em></h4>
        start a new task that will last for <em>duration</em>.
        <h4>!done <em>project-name</em> <em>task-name</em> <em>duration</em></h4>
        register a task that has lasted for <em>duration</em> from now.
        <h4>!switch <em>project-name</em> <em>task-name</em></em></h4>
            stop your current task and create a new one that has the same end time as the current one
        <h4>!stop</h4>
            stop your current task
        <h4>!more <em>duration</em></h4>
            stop your current task and create a 
new one with same project and task for <em>duration</em>
        <h4>!ls</h4>
            list current tasks
        <h4>!digest <em>project-name</em></h4>
            give stat for a given project
        <h4>!since <em>date-or-duration</em></h4>
            a summary of your tasks since date
        "
        .into(),
    ))
}
