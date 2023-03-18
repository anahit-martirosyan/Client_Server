use crate::context::Context;
use crate::utils::LocalError;
use http::header::{CONTENT_TYPE, LOCATION};
use http::request::Parts;
use http::{HeaderValue, StatusCode, Uri};
use hyper::{Body, Response};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

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

async fn get_json_from_body(body: Body) -> Option<Value> {
    let body_bytes = hyper::body::to_bytes(body).await.ok()?;

    let json: Option<Value> = serde_json::from_slice(&body_bytes).ok();

    json
}

// fn check_and_add_availability()

//
pub fn items_redirect(addr: String) -> Result<Response<Body>, hyper::Error> {
    create_redirect_response(StatusCode::PERMANENT_REDIRECT, addr)
}

//
pub fn response_ok() -> Result<Response<Body>, hyper::Error> {
    create_response(StatusCode::OK, String::new())
}

// /items
pub async fn get_items(context: Arc<Context>) -> Result<Response<Body>, hyper::Error> {
    match context.db.postgres_db.get_all_products().await {
        Err(error) => create_response(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
        Ok(items) => create_response(StatusCode::OK, items.to_string()),
    }
}

// /item
pub async fn get_item(
    parts: &Parts,
    context: Arc<Context>,
) -> Result<Response<Body>, hyper::Error> {
    let params = get_params(&parts.uri);
    let id = params.get("id");
    if id.is_none() {
        return create_response(StatusCode::BAD_REQUEST, LocalError::IdNotSent.to_string());
    }

    let id = id.unwrap().parse::<i32>();
    if id.is_err() {
        return create_response(StatusCode::BAD_REQUEST, LocalError::IdNotFound.to_string());
    }

    match context.db.postgres_db.get_product(id.unwrap()).await {
        Err(error) => {
            let status_code = if error == LocalError::IdNotFound {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            create_response(status_code, error.to_string())
        }
        Ok(item) => create_response(StatusCode::OK, item.to_string()),
    }
}

// /add_item
pub async fn add_item(body: Body, context: Arc<Context>) -> Result<Response<Body>, hyper::Error> {
    let mut json = get_json_from_body(body).await;

    let json_map: Option<&mut serde_json::Map<String, Value>> = json
        .as_mut()
        .and_then(|json: &mut Value| json.as_object_mut());

    if json_map.is_none() {
        return create_response(
            StatusCode::BAD_REQUEST,
            LocalError::WrongParameters.to_string(),
        );
    }

    let json_map = json_map.unwrap();

    if json_map.get("name").is_none()
        || json_map.get("price").is_none()
        || json_map.get("category").is_none()
    {
        return create_response(
            StatusCode::BAD_REQUEST,
            LocalError::WrongParameters.to_string(),
        );
    }

    json_map.entry("count").or_insert(json!(0));

    match context.db.postgres_db.add_product(json!(json_map)).await {
        Err(e) => {
            println!("{}", e.to_string());
            create_response(StatusCode::BAD_REQUEST, e.to_string())

        },
        Ok(id) => create_response(StatusCode::OK, id.to_string()),
    }
}

// /delete
pub async fn delete_item(
    parts: &Parts,
    context: Arc<Context>,
) -> Result<Response<Body>, hyper::Error> {
    let params = get_params(&parts.uri);
    let id = params.get("id");
    if id.is_none() {
        return create_response(StatusCode::BAD_REQUEST, LocalError::IdNotSent.to_string());
    }
    let id = id.unwrap().parse::<i32>();
    if id.is_err() {
        return create_response(StatusCode::BAD_REQUEST, LocalError::IdNotFound.to_string());
    }

    match context.db.postgres_db.delete_product(id.unwrap()).await {
        Err(error) => {
            create_response(StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
            // TODO check error types
        }
        Ok(_) => create_response(StatusCode::OK, String::new()),
    }
}

// /purchase
pub async fn buy_item(
    parts: &Parts,
    context: Arc<Context>,
) -> Result<Response<Body>, hyper::Error> {
    let params = get_params(&parts.uri);
    let id = params.get("id");
    if id.is_none() {
        return create_response(StatusCode::BAD_REQUEST, LocalError::IdNotSent.to_string());
    }
    let id = id.unwrap().parse::<i32>();
    if id.is_err() {
        return create_response(StatusCode::BAD_REQUEST, LocalError::IdNotFound.to_string());
    }
    let count: i32 = params
        .get("count")
        .and_then(|c| c.parse().ok())
        .unwrap_or(1);
    let user_id: Option<i32> = params.get("user_id").and_then(|c| c.parse().ok());

    // if user_id.is_none() {
    //     return create_response(
    //         StatusCode::BAD_REQUEST,
    //         LocalError::UnauthenticatedUser.to_string(),
    //     );
    // }

    match context
        .db
        .postgres_db
        .purchase(id.unwrap(), count, user_id.unwrap_or(1)
        )
        .await
    {
        Err(error) => create_response(StatusCode::BAD_REQUEST, error.to_string()),
        Ok(item) => create_response(StatusCode::OK, item.to_string()),
    }
}

// /add_account
pub async fn add_account(
    body: Body,
    context: Arc<Context>,
) -> Result<Response<Body>, hyper::Error> {
    let json = get_json_from_body(body).await;

    if json.is_none() {
        return create_response(
            StatusCode::BAD_REQUEST,
            LocalError::WrongParameters.to_string(),
        );
    }
    let json_map: Option<&serde_json::Map<String, Value>> =
        json.as_ref().and_then(|json: &Value| json.as_object());

    if json_map.is_none() {
        return create_response(
            StatusCode::BAD_REQUEST,
            LocalError::WrongParameters.to_string(),
        );
    }

    let json_map = json_map.unwrap();

    if json_map.get("username").is_none()
        || json_map.get("full_name").is_none()
        || json_map.get("password").is_none()
        || json_map.get("email").is_none()
        || json_map.get("phone").is_none()
    {
        return create_response(
            StatusCode::BAD_REQUEST,
            LocalError::WrongParameters.to_string(),
        );
    }

    match context.db.postgres_db.add_user(json.unwrap()).await {
        Err(e) => create_response(StatusCode::BAD_REQUEST, e.to_string()),
        Ok(id) => {
            let res = context.db.mongo_db.add_user(id).await;
            if res.is_err() {
                let _ = context.db.mongo_db.add_user(id).await;
            }

            create_response(StatusCode::OK, id.to_string())
        }
    }
}

pub async fn login(body: Body, context: Arc<Context>) -> Result<Response<Body>, hyper::Error> {
    let json = get_json_from_body(body).await;

    if json.is_none() {
        return create_response(
            StatusCode::BAD_REQUEST,
            LocalError::WrongParameters.to_string(),
        );
    }
    let json_map: Option<&serde_json::Map<String, Value>> =
        json.as_ref().and_then(|json: &Value| json.as_object());

    if json_map.is_none() {
        return create_response(
            StatusCode::BAD_REQUEST,
            LocalError::WrongParameters.to_string(),
        );
    }

    let json_map = json_map.unwrap();

    if json_map.get("username").is_none() || json_map.get("password").is_none() {
        return create_response(
            StatusCode::BAD_REQUEST,
            LocalError::WrongParameters.to_string(),
        );
    }

    match context.db.postgres_db.login(json.unwrap()).await {
        Err(e) => create_response(StatusCode::BAD_REQUEST, e.to_string()),
        Ok(id) => {
            let res = context.db.mongo_db.record_logged_in(id).await;
            if res.is_err() {
                let _ = context.db.mongo_db.record_logged_in(id).await;
            }

            create_response(StatusCode::OK, id.to_string())
        }
    }
}
