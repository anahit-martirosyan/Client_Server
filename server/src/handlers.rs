use http::header::{CONTENT_TYPE, LOCATION};
use http::request::Parts;
use http::{HeaderValue, StatusCode, Uri};
use hyper::{Body, Response};
use serde_json::Value;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use crate::items::Item;

static PRODUCTS_FILE: &str = "./data/products.json";

static ID_NOT_SENT: &str = "Wrong request";
static ID_NOT_FOUND: &str = "Wrong item";
static ITEM_NOT_AVAILABLE: &str = "Item is not available";
static PURCHASE_FAILED: &str = "Purchase has not been executed.";


fn create_response(status_code: StatusCode, body: String) -> Result<Response<Body>, hyper::Error> {
    Ok(Response::builder()
        .header(CONTENT_TYPE, "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Headers", "*")
        .header("Access-Control-Allow-Methods", "PUT, GET, OPTIONS")
        .status(status_code)
        .body(Body::from(body))
        .unwrap())
}

fn create_redirect_response(status_code: StatusCode, redirect_addres: String) -> Result<Response<Body>, hyper::Error> {
    Ok(Response::builder()
        .header(CONTENT_TYPE, "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .header(LOCATION, HeaderValue::from_str(&format!("http://{}/items", redirect_addres)).unwrap())
        .status(status_code)
        .body(Body::empty())
        .unwrap())
}

pub fn resonse_ok() -> Result<Response<Body>, hyper::Error> {
    create_response(StatusCode::OK, String::new())
}

pub fn get_params(uri: &Uri) -> HashMap<String, String> {
    uri.query()
        .map(|v| {
            url::form_urlencoded::parse(
                urlencoding::decode(v)
                    .unwrap_or_else(|_| v.to_string())
                    .as_bytes(),
            )
                .into_owned()
                .collect()
        })
        .unwrap_or_else(HashMap::new)
}

fn read_items() -> Vec<Item> {
    let json = std::fs::read_to_string(PRODUCTS_FILE).unwrap();
    let products = serde_json::from_str::<Value>(&json).unwrap();
    let mut items: Vec<Item> = vec![];
    for item in products.as_array().unwrap() {
        items.push(item.into());
    }

    items
}

fn purchase_item(items: &mut Vec<Item>, id: &str, count: i64) -> Result<Item, &'static str> {
    let item = items.iter_mut().find(|item| item.id == id);
    if item.is_none() {
        return Err(ID_NOT_FOUND);
    }
    if !item.as_ref().unwrap().is_available() {
        return Err(ITEM_NOT_AVAILABLE);
    }
    let mut item = item.unwrap();
    item.count -= count;

    let c = (*item).clone();

    if std::fs::write(PRODUCTS_FILE, serde_json::to_string_pretty(items).unwrap_or_default()).is_err() {
        return Err(PURCHASE_FAILED)
    }

    Ok(c)
}

pub fn items_redirect(addr: String) -> Result<Response<Body>, hyper::Error> {
    create_redirect_response(StatusCode::PERMANENT_REDIRECT, addr)
}

pub fn get_items() -> Result<Response<Body>, hyper::Error> {
    let products: Vec<Value> = read_items().into_iter().map(|item| Item::into(item)).collect();

    create_response(StatusCode::OK, serde_json::to_string(&products).unwrap())
}

pub fn buy_item(parts: &Parts) -> Result<Response<Body>, hyper::Error> {
    let params = get_params(&parts.uri);
    let id = params.get("id");
    let count: Option<i64> = params.get("count").and_then(|c| c.parse().ok());
    if id.is_none() {
        return create_response(StatusCode::BAD_REQUEST, ID_NOT_SENT.to_string())
    }
    let id = id.unwrap();
    let count = count.unwrap_or(1);

    let mut items = read_items();
    match purchase_item(items.borrow_mut(), id, count) {
         Err(status) => return create_response(StatusCode::BAD_REQUEST, status.to_string()),
         Ok(item) => {
             let v: Value = Item::into(item);
             return create_response(StatusCode::OK, v.to_string());
         }
    }
}
