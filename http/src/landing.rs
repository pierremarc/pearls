use html::{anchor, body, div, h2, head, html, span, style, title, with_doctype, Empty};
use warp::Filter;

fn document() -> String {
    let css = style(String::from(include_str!("landing.css"))).set("type", "text/css");
    let title = title(Empty).append_text("pearls");
    let content = div([
        h2("pearls"),
        span("A bot, counting time on"),
        anchor("matrix").set("href", "https://matrix.org"),
        span("network for"),
        anchor("atelier cartographique").set("href", "https://www.atelier-cartographique.be"),
        span("and friends."),
    ])
    .class("content");

    with_doctype(html([head([title, css]), body(content)]))
}

pub fn landing() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get().map(|| warp::reply::html(document()))
}
