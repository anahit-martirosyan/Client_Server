use crate::utils::LocalError;
use sea_orm::DbErr;

pub enum RecordType {
    User,
    Product,
    Order,
}

pub trait ToError<T> {
    fn to_local_error(self, record_type: RecordType) -> Result<T, LocalError>;
}

impl<T> ToError<T> for Result<T, DbErr> {
    fn to_local_error(self, record_type: RecordType) -> Result<T, LocalError> {
        match self {
            Ok(t) => Ok(t),
            Err(DbErr::RecordNotFound(e)) => {
                println!("{}", e);
                match record_type {
                    RecordType::Product | RecordType::Order => Err(LocalError::IdNotFound),
                    RecordType::User => Err(LocalError::WrongUserOrPassword),
                }
            }
            Err(DbErr::Json(e)) | Err(DbErr::Type(e)) => {
                println!("{}", e);
                Err(LocalError::WrongParameters)
            }
            Err(e) => {
                println!("{}", e);
                Err(LocalError::OperationFailed)
            }
        }
    }
}
