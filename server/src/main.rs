extern crate core;

use crate::context::Context;
use crate::settings::Settings;
use http::{Method, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use hyper::{http, Body, Request, Response, Server};
use std::net::SocketAddr;
use std::sync::Arc;

mod context;
mod db;
mod entities;
mod handlers;
mod settings;
mod utils;

async fn route_service(
    req: Request<Body>,
    addr: String,
    context: Arc<Context>,
) -> Result<Response<Body>, hyper::Error> {
    let (parts, body) = req.into_parts();

    println!("method: {}, uri: {}", &parts.method, parts.uri.path());

    match (&parts.method, parts.uri.path()) {
        (&Method::GET, "/") => handlers::items_redirect(addr),
        (&Method::GET, "/items") => handlers::get_items(context).await,
        (&Method::GET, "/item") => handlers::get_item(&parts, context).await,
        (&Method::POST, "/add_item") => handlers::add_item(body, context).await,
        (&Method::OPTIONS, "/add_item") => handlers::response_ok(),
        (&Method::DELETE, "/delete") => handlers::delete_item(&parts, context).await,
        (&Method::OPTIONS, "/purchase") => handlers::response_ok(),
        (&Method::PUT, "/purchase") => handlers::buy_item(&parts, context).await,
        (&Method::POST, "/add_account") => handlers::add_account(body, context).await,

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
    let settings = Settings::new("../config/config.yml");
    let db_url = settings.get("database", "url");
    let db_name = settings.get("database", "name");
    let context;
    match db::DB::init(&db_url, &db_name) {
        Err(e) => {
            println!("Error when initializing database: {}", e);
            return;
        }
        Ok(db) => {
            println!("Databse initialized: {}", db_name);
            context = Arc::new(Context { db });
        }
    };

    run_server(&settings, context.clone());
}
