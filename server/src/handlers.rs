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

fn get_id_from_uri(uri: &Uri) -> Result<i32, LocalError> {
    let params = get_params(uri);
    let id = params.get("id");
    if id.is_none() {
        return Err(LocalError::IdNotSent);
    }

    let id = id.unwrap().parse::<i32>();
    if id.is_err() {
        return Err(LocalError::IdNotFound);
    }

    Ok(id.unwrap())
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
    let id = get_id_from_uri(&parts.uri);

    if let Err(e) = id {
        return create_response(StatusCode::BAD_REQUEST, e.to_string());
    }

    let id = id.ok().unwrap();

    let res = context.cache.get_product(id);

    if let Some(item) = res {
        return create_response(StatusCode::OK, item.to_string());
    }

    match context.db.postgres_db.get_product(id).await {
        Err(error) => {
            let status_code = if error == LocalError::IdNotFound {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            create_response(status_code, error.to_string())
        }
        Ok(item) => {
            let _ = context.cache.add_product(id, item.clone());

            let user_id: Option<i32> = get_params(&parts.uri)
                .get("user_id")
                .and_then(|c| c.parse().ok());
            if let Some(user_id) = user_id {
                let res = context
                    .db
                    .mongo_db
                    .record_product_purchased(user_id, id)
                    .await;
                if res.is_err() {
                    let _ = context
                        .db
                        .mongo_db
                        .record_product_purchased(user_id, id)
                        .await;
                }
            }

            create_response(StatusCode::OK, item.to_string())
        }
    }
}

// /add_item
pub async fn add_item(
    mut body: Option<Value>,
    context: Arc<Context>,
) -> Result<Response<Body>, hyper::Error> {
    let json_map: Option<&mut serde_json::Map<String, Value>> = body
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
        }
        Ok(id) => create_response(StatusCode::OK, id.to_string()),
    }
}

// /update_item
pub async fn update_item(
    parts: &Parts,
    mut body: Option<Value>,
    context: Arc<Context>,
) -> Result<Response<Body>, hyper::Error> {
    let id = get_id_from_uri(&parts.uri);

    if let Err(e) = id {
        return create_response(StatusCode::BAD_REQUEST, e.to_string());
    }

    let id = id.ok().unwrap();

    let json_map: Option<&mut serde_json::Map<String, Value>> = body
        .as_mut()
        .and_then(|json: &mut Value| json.as_object_mut());

    if json_map.is_none() {
        return create_response(
            StatusCode::BAD_REQUEST,
            LocalError::WrongParameters.to_string(),
        );
    }

    let json_map = json_map.unwrap();

    let updates = json!(json_map);
    match context
        .db
        .postgres_db
        .update_product(id, updates.clone())
        .await
    {
        Err(e) => {
            println!("{}", e.to_string());
            create_response(StatusCode::BAD_REQUEST, e.to_string())
        }
        Ok(item) => {
            let res = context.cache.update_product(id.clone(), updates);
            if res.is_err() {
                let res = context.cache.delete_product(id.clone());
                if res.is_err() {
                    let _ = context.cache.delete_product(id);
                }
            }
            create_response(StatusCode::OK, item.to_string())
        }
    }
}

// /delete
pub async fn delete_item(
    parts: &Parts,
    context: Arc<Context>,
) -> Result<Response<Body>, hyper::Error> {
    let id = get_id_from_uri(&parts.uri);

    if let Err(e) = id {
        return create_response(StatusCode::BAD_REQUEST, e.to_string());
    }

    let id = id.ok().unwrap();

    match context.db.postgres_db.delete_product(id.clone()).await {
        Err(error) => {
            create_response(StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
            // TODO check error types
        }
        Ok(_) => {
            let res = context.cache.delete_product(id.clone());
            if res.is_err() {
                let _ = context.cache.delete_product(id.clone());
            }
            create_response(StatusCode::OK, String::new())
        }
    }
}

// /purchase
pub async fn buy_item(
    parts: &Parts,
    context: Arc<Context>,
) -> Result<Response<Body>, hyper::Error> {
    let id = get_id_from_uri(&parts.uri);

    if let Err(e) = id {
        return create_response(StatusCode::BAD_REQUEST, e.to_string());
    }

    let id = id.ok().unwrap();

    let params = get_params(&parts.uri);
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

    let user_id = user_id.unwrap_or(1);

    match context.db.postgres_db.purchase(id, count, user_id).await {
        Err(error) => create_response(StatusCode::BAD_REQUEST, error.to_string()),
        Ok(item) => {
            let res = context
                .db
                .mongo_db
                .record_product_purchased(user_id, id)
                .await;
            if res.is_err() {
                let _ = context
                    .db
                    .mongo_db
                    .record_product_purchased(user_id, id)
                    .await;
            }

            create_response(StatusCode::OK, item.to_string())
        }
    }
}

// /add_account
pub async fn add_account(
    body: Option<Value>,
    context: Arc<Context>,
) -> Result<Response<Body>, hyper::Error> {
    if body.is_none() {
        return create_response(
            StatusCode::BAD_REQUEST,
            LocalError::WrongParameters.to_string(),
        );
    }
    let json_map: Option<&serde_json::Map<String, Value>> =
        body.as_ref().and_then(|json: &Value| json.as_object());

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

    match context.db.postgres_db.add_user(body.unwrap()).await {
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

pub async fn login(
    body: Option<Value>,
    context: Arc<Context>,
) -> Result<Response<Body>, hyper::Error> {
    if body.is_none() {
        return create_response(
            StatusCode::BAD_REQUEST,
            LocalError::WrongParameters.to_string(),
        );
    }
    let json_map: Option<&serde_json::Map<String, Value>> =
        body.as_ref().and_then(|json: &Value| json.as_object());

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

    match context.db.postgres_db.login(body.unwrap()).await {
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
