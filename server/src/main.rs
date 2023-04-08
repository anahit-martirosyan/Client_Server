extern crate core;

use crate::cache::redis_cache;
use crate::context::Context;
use crate::settings::Settings;
use crate::utils::get_json_from_body;
use http::{Method, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use hyper::{http, Body, Request, Response, Server};
use std::net::SocketAddr;
use std::sync::Arc;

mod cache;
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

    let body_json = get_json_from_body(body).await;
    // if parts.method != Method::OPTIONS && body_json.is_some() {
    //     let _ = context.db.mongo_db.log_request(parts.uri.path(), body_json.as_ref().unwrap().clone());
    // }

    match (&parts.method, parts.uri.path()) {
        (&Method::GET, "/") => handlers::items_redirect(addr),
        (&Method::GET, "/items") => handlers::get_items(context).await,
        (&Method::GET, "/item") => handlers::get_item(&parts, context).await,
        (&Method::POST, "/add_item") => handlers::add_item(body_json, context).await,
        (&Method::OPTIONS, "/add_item") => handlers::response_ok(),
        (&Method::PUT, "/update_item") => handlers::update_item(&parts, body_json, context).await,
        (&Method::DELETE, "/delete") => handlers::delete_item(&parts, context).await,
        (&Method::OPTIONS, "/purchase") => handlers::response_ok(),
        (&Method::PUT, "/purchase") => handlers::buy_item(&parts, context).await,
        (&Method::POST, "/add_account") => handlers::add_account(body_json, context).await,
        (&Method::PUT, "/login") => handlers::login(body_json, context).await,
        // (&Method::PUT, "/logout") => handlers::logout(body, context).await,
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
    let mongodb_uri = settings.get("mongodb", "uri");
    let mongodb_name = settings.get("mongodb", "name");
    let redis_uri = settings.get("redis", "uri");

    let context;
    match db::DB::init(&postgres_url, &postgres_name, &mongodb_uri, &mongodb_name) {
        None => {
            println!("Error when initializing database");
            return;
        }
        Some(db) => {
            println!("Database initialized: {}", postgres_name);

            let cache = redis_cache::Cache::init(redis_uri);
            if cache.is_err() {
                println!("Error when initializing cache");
                return;
            }
            println!("Cache initialized");

            context = Arc::new(Context {
                db,
                cache: cache.unwrap(),
            });
        }
    };

    run_server(&settings, context.clone());
}
