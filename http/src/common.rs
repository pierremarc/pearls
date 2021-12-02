use shell::store::Store;
use std::sync::{Arc, Mutex};
use warp::Filter;

pub type ArcStore = Arc<Mutex<Store>>;

pub fn with_store(
    s: ArcStore,
) -> impl warp::Filter<Extract = (ArcStore,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || s.clone())
}

// fn with_token(
//     token: String,
// ) -> impl warp::Filter<Extract = (String,), Error = std::convert::Infallible> + Clone {
//     warp::any().map(move || token.clone())
// }

// fn with_base_path(
//     token: String,
// ) -> impl warp::Filter<Extract = (String,), Error = std::convert::Infallible> + Clone {
//     warp::any().map(move || format!("/{}/", token.clone()))
// }
