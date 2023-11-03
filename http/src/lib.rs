use shell::store::Store;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::{net::SocketAddr, path::Path};
use warp::Filter;

mod calendar;
mod common;
mod context;
mod helpers;
mod landing;
mod tabular;
mod timeline;
mod timeline2;
mod workload;

pub fn start_http(path: &Path, host: &str, static_dir: &str) {
    let addr: SocketAddr = host.parse().expect("Invalid address for the http server");
    let store = Store::new(String::from(path.to_string_lossy()));
    let arc_store = Arc::new(Mutex::new(store));

    let statics = warp::path("static").and(warp::fs::dir(PathBuf::from(static_dir)));

    let ctx = context::context(String::from(path.to_string_lossy()));

    std::thread::spawn(move || {
        let routes = calendar::calendar(arc_store.clone())
            .or(timeline::timeline(arc_store.clone()))
            .or(timeline2::timeline(ctx.clone()))
            .or(tabular::tabular(arc_store.clone()))
            .or(landing::room_landing(arc_store.clone()))
            .or(workload::workload(arc_store.clone()))
            .or(statics)
            .or(landing::landing(ctx.clone()));

        // let runtime = tokio::runtime::Runtime::new().expect("Failed to start tokio runtime");
        let mut runtime = tokio::runtime::Builder::new()
            .basic_scheduler()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(warp::serve(routes).run(addr));
    });
}
