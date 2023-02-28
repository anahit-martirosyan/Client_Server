#[derive(PartialEq)]
pub enum LocalError {
    IdNotSent,
    IdNotFound,
    ItemNotAvailable,
    WrongParameters,
    OperationFailed,
    UnauthenticatedUser,
}

impl LocalError {
    pub fn to_string(&self) -> String {
        match self {
            LocalError::IdNotSent => "Wrong request".to_string(),
            LocalError::IdNotFound => "Wrong item".to_string(),
            LocalError::ItemNotAvailable => "Item is not available".to_string(),
            LocalError::WrongParameters => "Wrong parameters".to_string(),
            LocalError::OperationFailed => "Operation has not been executed".to_string(),
            LocalError::UnauthenticatedUser => "User is not authenticated".to_string(),
        }
    }
}
