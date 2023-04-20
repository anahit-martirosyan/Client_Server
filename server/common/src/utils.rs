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

pub fn round(x: f64, decimals: u32) -> f64 {
    let y = 10i32.pow(decimals) as f64;
    (x * y).round() / y
}
