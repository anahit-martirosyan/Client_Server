extern crate core;

use crate::cache::redis_cache;
use crate::context::{Context, OrderManagerContext, UserManagerContext};
use common::request_response_utils::{get_json_from_body, response_ok, response_redirect};
use common::settings::Settings;
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

async fn route_service(
    req: Request<Body>,
    addr: String,
    context: Arc<Context>,
) -> Result<Response<Body>, hyper::Error> {
    let (parts, body) = req.into_parts();

    println!("method: {}, uri: {}", &parts.method, parts.uri.path());

    let body_json = get_json_from_body(body).await;

    match (&parts.method, parts.uri.path()) {
        (&Method::GET, "/") => response_redirect(addr),
        (&Method::GET, "/product/product/products") => handlers::get_items(context).await,
        (&Method::GET, "/product/product") => handlers::get_item(&parts, context).await,
        (&Method::POST, "/product/add") => handlers::add_item(body_json, context).await,
        (&Method::OPTIONS, "/product/add") => response_ok(),
        (&Method::PUT, "/product/update") => handlers::update_item(&parts, body_json, context).await,
        (&Method::DELETE, "/product/delete") => handlers::delete_item(&parts, context).await,
        (&Method::OPTIONS, "/product/purchase") => response_ok(),
        (&Method::PUT, "/product/purchase") => handlers::buy_item(&parts, context).await,
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

            let user_manager = UserManagerContext {
                uri: settings.get("user_manager", "uri"),
                product_viewed_endpoint: "/account/add_product_view".to_string(),
                product_purchased_endpoint: "/account/add_product_purchase".to_string()
            };

            let order_manager = OrderManagerContext {
                uri: settings.get("order_manager", "uri"),
                add_order_endpoint: "/order/add".to_string(),
            };

            context = Arc::new(Context {
                db,
                cache: cache.unwrap(),
                user_manager,
                order_manager,
            });
        }
    };

    run_server(&settings, context.clone());
}
