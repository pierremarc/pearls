use html::{anchor, div, em, h4, paragraph, span};

use crate::bot;

const DATE_HELP: &str = "
    Date must be formated as YYYY-MM-DD, DD-MM-YYY or DD-MM (for a date within the current year).
    The separatore can be either '-', '/' or '.'.
";

fn make_text(handler: &mut bot::CommandHandler) -> String {
    format!("
        !ping
            check if the bot's still alive
        !new <project-name>
            register a new project
        !deadline <project-name> <date>
            set a deadline for an existing project
            {}
        !provision <project-name> <duration>
            set provisioned time for an existing project
        !complete  <project-name> <date?> 
            set completion date for an existing project, if
            date is not provided, it will take the current time instead.
            {}
        !note <project-name> <text>
            add a note to a project, free text.
        !meta <project-name>
            mark this project as a meta project.
        !parent <child-project-name> <parent-project-name>
            attach a project to a meta project.
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

        A timeline is visible at http://{}/{}/timeline
        ", DATE_HELP, DATE_HELP, handler.host, handler.room_id)
}

fn make_html(handler: &mut bot::CommandHandler) -> String {
    div(vec![
        h4("!ping"),
        paragraph("check if the bot's still alive"),
        h4(vec![span("!new "), em("project-name")]),
        paragraph("register a new project"),
        h4(vec![
            span("!deadline  "),
            em("project-name "),
            em("date"),
        ]),
        paragraph("set a deadline for an existing project"),
        paragraph(DATE_HELP),
        h4(vec![
            span("!provision  "),
            em("project-name "),
            em("duration"),
        ]),
        paragraph("set provisioned time for an existing project"),
        h4(vec![span("!complete   "), em("project-name "), em("date?")]),
        paragraph(
            "set completion date for an existing project, if
            date is not provided, it will take the current time instead.",
        ),
        paragraph(DATE_HELP),
        h4(vec![span("!note   "),  em("project-name "), em("text")]),
        paragraph(
            "add a note to a project, free text.",
        ),
        h4(vec![span("!meta   "),  em("project-name ")]),
        paragraph(
            "mark this project as a meta project.",
        ),
        h4(vec![span("!parent   "),  em("child-project-name "),  em("parent-project-name ")]),
        paragraph(
            "attach a project to a meta project.",
        ),
        h4(vec![
            span("!do  "),
            em("project-name "),
            em("task "),
            em("duration "),
        ]),
        paragraph("start a new task that will last for <em>duration</em>."),
        h4(vec![
            span("!done  "),
            em("project-name "),
            em("task "),
            em("duration "),
        ]),
        paragraph("register a task that has lasted for <em>duration</em> from now."),
        h4(vec![
            span("!switch  "),
            em("project-name "),
            em("task "),
        ]),
        paragraph("stop your current task and create a new one that has the same end time as the current one."),
        h4(vec![
            span("!stop  "),
            ]),
        paragraph("stop your current task."),
        h4(vec![
            span("!more  "),
            em("duration "),
            ]),
        paragraph("stop your current task and create a new one with same project and task for <em>duration</em>."),
        h4(vec![
            span("!ls  "),
            ]),
        paragraph("list current tasks."),
        h4(vec![
            span("!digest  "),
            em("project-name "),
        ]),
        paragraph("give stats for a given project."),
        h4(vec![
            span("!since  "),
            em("duration "),
        ]),
        paragraph("a summary of your tasks since date."),
        div(vec![
            anchor("TIMELINE").set("href", format!("http://{}/{}/timeline
        ", handler.host, handler.room_id))
        ])
        ])
        .as_string()
}

pub fn help(handler: &mut bot::CommandHandler) -> Option<(String, String)> {
    Some((make_text(handler), make_html(handler)))
}
