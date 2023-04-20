use crate::context::Context;
use common::request_response_utils::{create_response, get_id_from_uri};
use common::utils::LocalError;
use http::request::Parts;
use http::StatusCode;
use hyper::{Body, Response};
use serde_json::Value;
use std::sync::Arc;

// /account/add
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

// /account/login
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

// /account/add_product_view
pub async fn add_product_view(
    parts: &Parts,
    context: Arc<Context>,
) -> Result<Response<Body>, hyper::Error> {
    let user_id = get_id_from_uri(&parts.uri, "user_id");
    if let Err(e) = user_id {
        return create_response(StatusCode::BAD_REQUEST, e.to_string());
    }

    let product_id = get_id_from_uri(&parts.uri, "product_id");
    if let Err(e) = product_id {
        return create_response(StatusCode::BAD_REQUEST, e.to_string());
    }

    let user_id = user_id.ok().unwrap();
    let product_id = product_id.ok().unwrap();

    let mut res = context
        .db
        .mongo_db
        .record_product_viewed(user_id, product_id)
        .await;

    if res.is_err() {
        res = context
            .db
            .mongo_db
            .record_product_viewed(user_id, product_id)
            .await;
    }

    if res.is_err() {
        create_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            LocalError::OperationFailed.to_string(),
        )
    } else {
        create_response(StatusCode::OK, String::new())
    }
}

// /account/add_product_purchase
pub async fn add_product_purchase(
    parts: &Parts,
    context: Arc<Context>,
) -> Result<Response<Body>, hyper::Error> {
    let user_id = get_id_from_uri(&parts.uri, "user_id");
    if let Err(e) = user_id {
        return create_response(StatusCode::BAD_REQUEST, e.to_string());
    }

    let product_id = get_id_from_uri(&parts.uri, "product_id");
    if let Err(e) = product_id {
        return create_response(StatusCode::BAD_REQUEST, e.to_string());
    }

    let user_id = user_id.ok().unwrap();
    let product_id = product_id.ok().unwrap();

    let mut res = context
        .db
        .mongo_db
        .record_product_purchased(user_id, product_id)
        .await;

    if res.is_err() {
        res = context
            .db
            .mongo_db
            .record_product_purchased(user_id, product_id)
            .await;
    }

    if res.is_err() {
        create_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            LocalError::OperationFailed.to_string(),
        )
    } else {
        create_response(StatusCode::OK, String::new())
    }
}
