use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde_json::Value;

use crate::entities::account;
use common::utils::LocalError;

use common::db_utils::{RecordType, ToError};

pub struct PostgresDB {
    pub db: DatabaseConnection,
}

impl PostgresDB {
    pub async fn add_user(&self, user_json: Value) -> Result<i32, LocalError> {
        let new_account =
            account::ActiveModel::from_json(user_json).to_local_error(RecordType::User)?;

        let res = account::Entity::insert(new_account)
            .exec(&self.db)
            .await
            .to_local_error(RecordType::User)?;
        Ok(res.last_insert_id)
    }

    pub async fn login(&self, user_json: Value) -> Result<i32, LocalError> {
        let username = user_json
            .get("username")
            .unwrap()
            .as_str()
            .unwrap_or_default();
        let password = user_json
            .get("password")
            .unwrap()
            .as_str()
            .unwrap_or_default();

        let user: Option<account::Model> = account::Entity::find()
            .filter(account::Column::Username.eq(username))
            .one(&self.db)
            .await
            .to_local_error(RecordType::User)?;

        if let Some(user) = user {
            if &user.password == password {
                return Ok(user.user_id);
            }
        }

        Err(LocalError::WrongUserOrPassword)
    }
}
