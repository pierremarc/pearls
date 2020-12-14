use shell::store::Store;
use std::sync::{Arc, Mutex};
use std::{net::SocketAddr, path::Path};
use warp::Filter;

mod calendar;
mod timeline;

type ArcStore = Arc<Mutex<Store>>;

fn with_store(
    s: ArcStore,
) -> impl warp::Filter<Extract = (ArcStore,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || s.clone())
}

fn with_token(
    token: String,
) -> impl warp::Filter<Extract = (String,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || token.clone())
}

fn with_base_path(
    token: String,
) -> impl warp::Filter<Extract = (String,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || format!("/{}/", token.clone()))
}

fn check_token(token: String) -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    warp::any()
        .and(with_token(token))
        .and(warp::path::param())
        .and_then(|token: String, prefix: String| async move {
            if prefix == token {
                Ok(())
            } else {
                Err(warp::reject())
            }
        })
        .untuple_one()
}

pub fn start_http(path: &Path, host: &str, token: &str) {
    let addr: SocketAddr = host.parse().expect("Invalid address");
    let store = Store::new(path.clone()).expect("Failed to get a store for HTTP server");
    let arc_store = Arc::new(Mutex::new(store));
    let token_string = String::from(token);

    std::thread::spawn(move || {
        let routes = check_token(token_string.clone()).and(
            calendar::calendar(arc_store.clone())
                .or(timeline::timeline(arc_store.clone(), token_string.clone())),
        );
        // let runtime = tokio::runtime::Runtime::new().expect("Failed to start tokio runtime");
        let mut runtime = tokio::runtime::Builder::new()
            .basic_scheduler()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(warp::serve(routes).run(addr));
    });
}
