use handlebars::{Handlebars, RenderError};
use serde::Serialize;
use shell::store::Store;
use std::file;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct Context<'a> {
    store: Store,
    registry: Handlebars<'a>,
}

impl<'a> Context<'a> {
    pub fn render<T>(&self, name: &str, data: &T) -> Result<String, RenderError>
    where
        T: Serialize,
    {
        self.registry.render(name, data)
    }
}

pub type ArcContext<'a> = Arc<Mutex<Context<'a>>>;

pub fn context<'a>(path: String) -> ArcContext<'a> {
    let mut registry = Handlebars::new();

    #[cfg(debug_assertions)]
    registry.set_dev_mode(true);

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
