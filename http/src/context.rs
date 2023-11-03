use handlebars::{Handlebars, RenderError};
use serde::Serialize;
use shell::store::{ConnectedStore, Store};
use std::file;
use std::path::Path;
use std::sync::{Arc, Mutex};
use warp::Filter;

use crate::helpers::register_helpers;

pub struct Context<'a> {
    store: Store,
    registry: Handlebars<'a>,
}

impl<'a> Context<'a> {
    pub fn render<T>(&self, template_name: &str, data: &T) -> Result<String, RenderError>
    where
        T: Serialize,
    {
        self.registry.render(template_name, data)
    }

    pub fn render_with<P, T>(
        &mut self,
        template_name: &str,
        db_name: &str,
        provider: P,
    ) -> Result<String, RenderError>
    where
        T: Serialize,
        P: Fn(&mut ConnectedStore) -> T,
    {
        if let Ok(connected) = self.store.connect_existing(&db_name) {
            let data = provider(connected);
            self.registry.render(template_name, &data)
        } else {
            Err(RenderError::new(format!(
                "Database {} does not exist",
                db_name
            )))
        }
    }
}

pub type ArcContext<'a> = Arc<Mutex<Context<'a>>>;

pub fn context<'a>(path: String) -> ArcContext<'a> {
    let mut registry = Handlebars::new();

    #[cfg(debug_assertions)]
    registry.set_dev_mode(true);

    register_helpers(&mut registry);

    let _ = registry
        .register_templates_directory(
            ".html",
            Path::new(file!()).parent().unwrap().join("templates"),
        )
        .unwrap();

    Arc::new(Mutex::new(Context {
        store: Store::new(path),
        registry,
    }))
}

pub fn with_context(
    ctx: ArcContext<'_>,
) -> impl warp::Filter<Extract = (ArcContext,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || ctx.clone())
}
