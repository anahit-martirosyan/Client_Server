use crate::context::Context;
use chrono::Utc;
use common::request_response_utils::create_response;
use common::utils::LocalError;
use http::StatusCode;
use hyper::{Body, Response};
use serde_json::{json, Value};
use std::sync::Arc;

// /order/add
pub async fn add_order(
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

    if json_map.get("user_id").is_none()
        || json_map.get("product_id").is_none()
        || json_map.get("total_price").is_none()
    {
        return create_response(
            StatusCode::BAD_REQUEST,
            LocalError::WrongParameters.to_string(),
        );
    }

    json_map
        .entry("date_time")
        .or_insert(json!(Utc::now().naive_utc()));

    match context.db.postgres_db.add_order(json!(json_map)).await {
        Err(e) => {
            println!("{}", e.to_string());
            create_response(StatusCode::BAD_REQUEST, e.to_string())
        }
        Ok(id) => create_response(StatusCode::OK, id.to_string()),
    }
}
