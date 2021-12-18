use shell::store::Store;
use std::sync::{Arc, Mutex};
use std::{net::SocketAddr, path::Path};
use warp::Filter;

mod calendar;
mod common;
mod landing;
mod tabular;
mod timeline;

pub fn start_http(path: &Path, host: &str) {
    let addr: SocketAddr = host.parse().expect("Invalid address for the http server");
    let store = Store::new(String::from(path.to_string_lossy()));
    let arc_store = Arc::new(Mutex::new(store));

    std::thread::spawn(move || {
        let routes = calendar::calendar(arc_store.clone())
            .or(timeline::timeline(arc_store.clone()))
            .or(tabular::tabular(arc_store.clone()))
            .or(landing::room_landing(arc_store.clone()))
            .or(landing::landing());
        // let runtime = tokio::runtime::Runtime::new().expect("Failed to start tokio runtime");
        let mut runtime = tokio::runtime::Builder::new()
            .basic_scheduler()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(warp::serve(routes).run(addr));
    });
}
