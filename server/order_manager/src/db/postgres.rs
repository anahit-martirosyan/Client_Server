use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};
use serde_json::Value;

use crate::entities::orders;

use common::db_utils::{RecordType, ToError};
use common::utils::LocalError;

pub struct PostgresDB {
    pub db: DatabaseConnection,
}

impl PostgresDB {
    pub async fn add_order(&self, order_json: Value) -> Result<i32, LocalError> {
        let order = orders::ActiveModel::from_json(order_json).to_local_error(RecordType::Order)?;
        let res = orders::Entity::insert(order)
            .exec(&self.db)
            .await
            .to_local_error(RecordType::Order)?;

        Ok(res.last_insert_id)
    }
}
