use crate::bot;
use std::time;

pub fn stop(handler: &mut bot::Context, user: String) -> Option<(String, String)> {
    let pendings = handler
        .store
        .select_current_task_for(user).unwrap_or_default();
    let pending = pendings.first();
    match pending {
        Some(rec) => match handler
            .store
            .update_task_end(rec.id, time::SystemTime::now())
        {
            Err(_) => None,
            Ok(_) => Some((
                "Done, you can !do a new one".into(),
                "Done, you can <strong>!do</strong> a new one".into(),
            )),
        },
        None => Some((
            String::from("Ther's nothing to !stop for you"),
            String::new(),
        )),
    }
}
