extern crate core;

use crate::context::Context;
use common::request_response_utils::{get_json_from_body, response_ok};
use common::settings::Settings;
use http::{Method, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use hyper::{http, Body, Request, Response, Server};
use std::net::SocketAddr;
use std::sync::Arc;

mod context;
mod db;
mod entities;
mod handlers;

async fn route_service(
    req: Request<Body>,
    _addr: String,
    context: Arc<Context>,
) -> Result<Response<Body>, hyper::Error> {
    let (parts, body) = req.into_parts();

    println!("method: {}, uri: {}", &parts.method, parts.uri.path());

    let body_json = get_json_from_body(body).await;

    match (&parts.method, parts.uri.path()) {
        (&Method::POST, "/order/add") => handlers::add_order(body_json, context).await,
        (&Method::OPTIONS, "/order/add") => response_ok(),
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()),
    }
}

#[tokio::main]
pub async fn run_server(settings: &Settings, context: Arc<Context>) {
    let addr = settings.get("network", "listen_on");
    let in_addr: SocketAddr = addr.parse().unwrap();
    println!("{}", settings.get("network", "listen_on"));

    let service = make_service_fn(move |_| {
        let addr = addr.clone();
        let context = context.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                route_service(req, addr.clone(), context.clone())
            }))
        }
    });

    let server = Server::bind(&in_addr).serve(service);

    if let Err(e) = server.await {
        println!("server error: {}", e);
    } else {
        println!("Listening on http://{}", in_addr);
    }
}

fn main() {
    let settings = Settings::new("./config/config.yml");
    let postgres_url = settings.get("postgres", "uri");
    let postgres_name = settings.get("postgres", "name");

    let context;
    match db::DB::init(&postgres_url, &postgres_name) {
        None => {
            println!("Error when initializing database");
            return;
        }
        Some(db) => {
            println!("Database initialized: {}", postgres_name);

            context = Arc::new(Context { db });
        }
    };

    run_server(&settings, context.clone());
}
