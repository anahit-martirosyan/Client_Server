use hyper::Body;
use serde_json::Value;

#[derive(PartialEq)]
pub enum LocalError {
    IdNotSent,
    IdNotFound,
    ItemNotAvailable,
    WrongParameters,
    OperationFailed,
    UnauthenticatedUser,
    WrongUserOrPassword,
}

impl LocalError {
    pub fn to_string(&self) -> String {
        match self {
            LocalError::IdNotSent => "Wrong request".to_string(),
            LocalError::IdNotFound => "Wrong ID".to_string(),
            LocalError::ItemNotAvailable => "Item is not available".to_string(),
            LocalError::WrongParameters => "Wrong parameters".to_string(),
            LocalError::OperationFailed => "Operation has not been executed".to_string(),
            LocalError::UnauthenticatedUser => "User is not authenticated".to_string(),
            LocalError::WrongUserOrPassword => "Wrong user or password".to_string(),
        }
    }
}


pub async fn get_json_from_body(body: Body) -> Option<Value> {
    let body_bytes = hyper::body::to_bytes(body).await.ok()?;

    let json: Option<Value> = serde_json::from_slice(&body_bytes).ok();

    json
}