use shell::store::Store;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
// use tokio::prelude::*;
use tower_web::ServiceBuilder;

#[derive(Clone)]
struct Cal {
    pub store: Arc<Mutex<Store>>,
}

impl_web! {
    impl Cal {
        #[get("/cal/:uuid")]
        #[content_type("html")]
        fn cal(&self, uuid: String) -> Result<String, ()> {
            let store = self.store.lock().expect("This mutex should not hold");
            match store.select_cal(uuid) {
                Ok(cal) => Ok(cal.content),
                Err(_) => Err(()),
            }
        }
    }
}

pub fn start_http(path: &Path, host: &str) {
    let addr = host.parse().expect("Invalid address");
    println!("Listening on http://{}", addr);
    match Store::new(path.clone()) {
        Ok(store) => {
            thread::spawn(move || {
                ServiceBuilder::new()
                    .resource(Cal {
                        store: Arc::new(Mutex::new(store)),
                    })
                    .run(&addr)
                    .unwrap();
            });
        }
        Err(err) => {
            println!("Could not start the http server:\n\t{}", err);
        }
    };
}
