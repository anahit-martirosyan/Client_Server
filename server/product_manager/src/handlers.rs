use std::collections::HashMap;
use crate::context::Context;
use common::request_response_utils::*;
use common::utils::LocalError;
use http::request::Parts;
use http::StatusCode;
use hyper::{Body, Client, Response};
use hyper_tls::HttpsConnector;
use serde_json::{json, Value};
use std::sync::Arc;
use url::Url;

// /product/product/products
pub async fn get_items(context: Arc<Context>) -> Result<Response<Body>, hyper::Error> {
    match context.db.postgres_db.get_all_products().await {
        Err(error) => create_response(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
        Ok(items) => create_response(StatusCode::OK, items.to_string()),
    }
}

// /product/product
pub async fn get_item(
    parts: &Parts,
    context: Arc<Context>,
) -> Result<Response<Body>, hyper::Error> {
    let id = get_id_from_uri(&parts.uri, "id");

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

            let res = context.db.mongo_db.record_product_viewed(id).await;
            if res.is_err() {
                let _ = context.db.mongo_db.record_product_viewed(id).await;
            }

            let user_id = get_id_from_uri(&parts.uri, "user_id");

            if let Ok(user_id) = user_id {
                // add product viewed for user
                let url = Url::parse(&format!("{}{}", context.user_manager.uri, context.user_manager.product_viewed_endpoint));
                if let Ok(mut url) = url {
                    url.query_pairs_mut().append_pair("user_id", &user_id.to_string());
                    url.query_pairs_mut().append_pair("product_id", &id.to_string());

                    let request = hyper::Request::get(url.as_str())
                        .body(Body::empty());

                    if let Ok(request) = request {
                        let https = HttpsConnector::new();
                        let client = Client::builder().build::<_, hyper::Body>(https);
                        let _response = client.request(request).await;
                    }
                };
            }

            create_response(StatusCode::OK, item.to_string())
        }
    }
}

// /product/add
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

// /product/update
pub async fn update_item(
    parts: &Parts,
    mut body: Option<Value>,
    context: Arc<Context>,
) -> Result<Response<Body>, hyper::Error> {
    let id = get_id_from_uri(&parts.uri, "id");

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
    let id = get_id_from_uri(&parts.uri, "id");

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

// /product/purchase
pub async fn buy_item(
    parts: &Parts,
    context: Arc<Context>,
) -> Result<Response<Body>, hyper::Error> {
    let id = get_id_from_uri(&parts.uri, "id");

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

    match context.db.postgres_db.purchase(id, count).await {
        Err(error) => create_response(StatusCode::BAD_REQUEST, error.to_string()),
        Ok(item) => {
            let res = context.db.mongo_db.record_product_purchased(id).await;
            if res.is_err() {
                let _ = context.db.mongo_db.record_product_purchased(id).await;
            }

            // add product purchased for user
            let url = Url::parse(&format!("{}{}", context.user_manager.uri, context.user_manager.product_purchased_endpoint));
            if let Ok(mut url) = url {
                url.query_pairs_mut().append_pair("user_id", &user_id.to_string());
                url.query_pairs_mut().append_pair("product_id", &id.to_string());

                let request = hyper::Request::get(url.as_str())
                    .body(Body::empty());

                if let Ok(request) = request {
                    let https = HttpsConnector::new();
                    let client = Client::builder().build::<_, hyper::Body>(https);
                    let _response = client.request(request).await;
                }
            };

            // add order
            let url = Url::parse(&format!("{}{}", context.order_manager.uri, context.order_manager.add_order_endpoint));
            if let Ok(url) = url {
                let mut request_params = HashMap::new();
                request_params.insert("user_id", json!(user_id));
                request_params.insert("product_id", json!(id));
                request_params.insert("total_price", json!(item.get("price").unwrap()));
                let request = hyper::Request::get(url.as_str())
                    .body(Body::from(serde_json::to_string(&request_params).unwrap()));

                if let Ok(request) = request {
                    let https = HttpsConnector::new();
                    let client = Client::builder().build::<_, hyper::Body>(https);
                    let _response = client.request(request).await;
                }
            };

            create_response(StatusCode::OK, item.to_string())
        }
    }
}
