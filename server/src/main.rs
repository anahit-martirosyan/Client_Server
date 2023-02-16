extern crate core;

use crate::settings::Settings;
use http::{Method, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use hyper::{http, Body, Request, Response, Server};
use std::net::SocketAddr;

mod handlers;
mod items;
mod settings;
mod db;
mod utils;

async fn route_service(req: Request<Body>, addr: String) -> Result<Response<Body>, hyper::Error> {
    let (parts, body) = req.into_parts();

    println!("method: {}, uri: {}", &parts.method, parts.uri.path());

    match (&parts.method, parts.uri.path()) {
        (&Method::GET, "/") => handlers::items_redirect(addr),
        (&Method::GET, "/items") => handlers::get_items(),
        (&Method::GET, "/item") => handlers::get_item(&parts),
        (&Method::POST, "/add") => handlers::add_item(body).await,
        (&Method::OPTIONS, "/add") => handlers::response_ok(),
        (&Method::DELETE, "/delete") => handlers::delete_item(&parts),
        (&Method::OPTIONS, "/purchase") => handlers::response_ok(),
        (&Method::PUT, "/purchase") => handlers::buy_item(&parts),

        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()),
    }
}

#[tokio::main]
pub async fn run_server(settings: &Settings) {
    let addr = settings.get("network", "listen_on");
    let in_addr: SocketAddr = addr.parse().unwrap();
    println!("{}", settings.get("network", "listen_on"));

    let service = make_service_fn(move |_| {
        let addr = addr.clone();
        async move { Ok::<_, hyper::Error>(service_fn(move |req| route_service(req, addr.clone()))) }
    });

    let server = Server::bind(&in_addr).serve(service);

    if let Err(e) = server.await {
        println!("server error: {}", e);
    } else {
        println!("Listening on http://{}", in_addr);
    }
}

fn main() {
    let settings = Settings::new("../config/config.yml");

    run_server(&settings);
}
