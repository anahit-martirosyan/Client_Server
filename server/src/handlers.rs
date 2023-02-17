use crate::db;
use crate::items::Item;
use crate::utils;
use http::header::{CONTENT_TYPE, LOCATION};
use http::request::Parts;
use http::{HeaderValue, StatusCode, Uri};
use hyper::{Body, Response};
use serde_json::{Map, Value};
use std::collections::HashMap;

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

fn create_redirect_response(
    status_code: StatusCode,
    redirect_addres: String,
) -> Result<Response<Body>, hyper::Error> {
    Ok(Response::builder()
        .header(CONTENT_TYPE, "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .header(
            LOCATION,
            HeaderValue::from_str(&format!("http://{}/items", redirect_addres)).unwrap(),
        )
        .status(status_code)
        .body(Body::empty())
        .unwrap())
}

fn get_params(uri: &Uri) -> HashMap<String, String> {
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

async fn get_json_from_body(body: Body) -> Option<Map<String, Value>> {
    let body_bytes = hyper::body::to_bytes(body).await.ok()?;

    let json: Option<Value> = serde_json::from_slice(&body_bytes).ok();
    let json: Option<&serde_json::Map<String, Value>> =
        json.as_ref().and_then(|json: &Value| json.as_object());

    json.cloned()
}

//
pub fn items_redirect(addr: String) -> Result<Response<Body>, hyper::Error> {
    create_redirect_response(StatusCode::PERMANENT_REDIRECT, addr)
}

//
pub fn response_ok() -> Result<Response<Body>, hyper::Error> {
    create_response(StatusCode::OK, String::new())
}

// /items
pub fn get_items() -> Result<Response<Body>, hyper::Error> {
    let items = db::read_items();
    match items {
        Err(error) => create_response(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
        Ok(items) => {
            let items: Vec<Value> = items
                .into_iter()
                .map(|(_, item)| Item::into(item))
                .collect();

            create_response(StatusCode::OK, serde_json::to_string(&items).unwrap())
        }
    }
}

// /item
pub fn get_item(parts: &Parts) -> Result<Response<Body>, hyper::Error> {
    let params = get_params(&parts.uri);
    let id = params.get("id");
    if id.is_none() {
        return create_response(StatusCode::BAD_REQUEST, utils::ID_NOT_SENT.to_string());
    }

    match db::get_item(id.unwrap()) {
        Err(error) => {
            let status_code = if error == utils::ID_NOT_FOUND {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            create_response(status_code, error.to_string())
        }
        Ok(item) => create_response(StatusCode::OK, serde_json::to_string(&item).unwrap()),
    }
}

// /add
pub async fn add_item(body: Body) -> Result<Response<Body>, hyper::Error> {
    let json = get_json_from_body(body).await;

    if json.is_none() {
        return create_response(StatusCode::BAD_REQUEST, utils::WRONG_PARAMETERS.to_string());
    }
    let json = json.unwrap();

    if json.get("name").is_none() || json.get("price").is_none() || json.get("category").is_none() {
        return create_response(StatusCode::BAD_REQUEST, utils::WRONG_PARAMETERS.to_string());
    }

    let name = json.get("name").unwrap().to_string();
    let price = json.get("price").unwrap().as_f64().unwrap_or_default();
    let category = json.get("category").unwrap().to_string();
    let count = json.get("count").and_then(|c| c.as_i64()).unwrap_or(0);
    let image = json
        .get("image")
        .and_then(|i| i.as_str())
        .unwrap_or("")
        .to_string();

    let new_id = db::get_new_id();
    if let Err(error) = new_id {
        return create_response(StatusCode::INTERNAL_SERVER_ERROR, error.to_string());
    }

    let new_item = Item {
        id: new_id.as_ref().unwrap().clone(),
        name,
        image,
        count,
        price,
        category,
    };

    if let Err(error) = db::add_item(new_item) {
        return create_response(StatusCode::INTERNAL_SERVER_ERROR, error.to_string());
    }

    create_response(StatusCode::OK, new_id.unwrap())
}

// /delete
pub fn delete_item(parts: &Parts) -> Result<Response<Body>, hyper::Error> {
    let params = get_params(&parts.uri);
    let id = params.get("id");
    if id.is_none() {
        return create_response(StatusCode::BAD_REQUEST, utils::ID_NOT_SENT.to_string());
    }

    match db::delete_item(id.unwrap()) {
        Err(error) => {
            let status_code = if error == utils::ID_NOT_FOUND {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            create_response(status_code, error.to_string())
        }
        Ok(_) => create_response(StatusCode::OK, String::new()),
    }
}

// /purchase
pub fn buy_item(parts: &Parts) -> Result<Response<Body>, hyper::Error> {
    let params = get_params(&parts.uri);
    let id = params.get("id");
    let count: Option<i64> = params.get("count").and_then(|c| c.parse().ok());
    if id.is_none() {
        return create_response(StatusCode::BAD_REQUEST, utils::ID_NOT_SENT.to_string());
    }
    let id = id.unwrap();
    let count = count.unwrap_or(1);

    match db::purchase_item(id, count) {
        Err(error) => create_response(StatusCode::BAD_REQUEST, error.to_string()),
        Ok(item) => create_response(StatusCode::OK, serde_json::to_string(&item).unwrap()),
    }
}
