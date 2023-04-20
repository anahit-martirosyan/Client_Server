use crate::utils::LocalError;
use http::header::{CONTENT_TYPE, LOCATION};
use http::{HeaderValue, StatusCode, Uri};
use hyper::{Body, Response};
use serde_json::Value;
use std::collections::HashMap;

pub fn create_response(
    status_code: StatusCode,
    body: String,
) -> Result<Response<Body>, hyper::Error> {
    Ok(Response::builder()
        .header(CONTENT_TYPE, "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Headers", "*")
        .header("Access-Control-Allow-Methods", "PUT, GET, OPTIONS")
        .status(status_code)
        .body(Body::from(body))
        .unwrap())
}

pub fn create_redirect_response(
    status_code: StatusCode,
    redirect_addres: String,
) -> Result<Response<Body>, hyper::Error> {
    Ok(Response::builder()
        .header(CONTENT_TYPE, "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .header(
            LOCATION,
            HeaderValue::from_str(&format!("http://{}/product/product/products", redirect_addres)).unwrap(),
        )
        .status(status_code)
        .body(Body::empty())
        .unwrap())
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

pub fn get_id_from_uri(uri: &Uri, id_name: &str) -> Result<i32, LocalError> {
    let params = get_params(uri);
    let id = params.get(id_name);
    if id.is_none() {
        return Err(LocalError::IdNotSent);
    }

    let id = id.unwrap().parse::<i32>();
    if id.is_err() {
        return Err(LocalError::IdNotFound);
    }

    Ok(id.unwrap())
}

//
pub fn response_redirect(addr: String) -> Result<Response<Body>, hyper::Error> {
    create_redirect_response(StatusCode::PERMANENT_REDIRECT, addr)
}

//
pub fn response_ok() -> Result<Response<Body>, hyper::Error> {
    create_response(StatusCode::OK, String::new())
}

pub async fn get_json_from_body(body: Body) -> Option<Value> {
    let body_bytes = hyper::body::to_bytes(body).await.ok()?;

    let json: Option<Value> = serde_json::from_slice(&body_bytes).ok();

    json
}
