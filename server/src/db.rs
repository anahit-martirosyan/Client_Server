use sea_orm::{
    ActiveModelTrait, ActiveValue, ConnectionTrait, Database, DatabaseConnection, DbErr,
    EntityTrait, Statement,
};
use serde_json::{json, Value};

use crate::entities::{account, orders, product};
use crate::utils::LocalError;

use chrono::Utc;

trait ToError<T> {
    fn to_local_error(self) -> Result<T, LocalError>;
}

impl<T> ToError<T> for Result<T, DbErr> {
    fn to_local_error(self) -> Result<T, LocalError> {
        match self {
            Ok(t) => Ok(t),
            Err(DbErr::RecordNotFound(_)) => Err(LocalError::IdNotFound),
            Err(DbErr::Json(_)) | Err(DbErr::Type(_)) => Err(LocalError::WrongParameters),
            Err(_) => Err(LocalError::OperationFailed),
        }
    }
}

pub struct DB {
    db: DatabaseConnection,
}

impl DB {
    #[tokio::main]
    pub async fn init(db_url: &str, db_name: &str) -> Result<DB, DbErr> {
        let statement = format!("SELECT \'CREATE DATABASE {}\' WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = \'{}\')", db_name, db_name);
        println!("{}", statement);
        let db = Database::connect(db_url).await?;
        db.execute(Statement::from_string(db.get_database_backend(), statement))
            .await?;

        let url = format!("{}/{}", db_url, db_name);
        let db_con = Database::connect(&url).await?;

        Ok(DB { db: db_con })
    }

    pub async fn add_user(&self, user_json: Value) -> Result<i32, LocalError> {
        // let username = user_json.get("username").unwrap().to_string();
        // let full_name = user_json.get("full_name").unwrap().to_string();
        // let password = user_json.get("password").unwrap().to_string();
        // let email = user_json.get("email").unwrap().to_string();
        // let phone = user_json.get("phone").unwrap().to_string();

        let new_account = account::ActiveModel::from_json(user_json).to_local_error()?;
        //     account::ActiveModel {
        //     username: ActiveValue::Set(username),
        //     full_name: ActiveValue::Set(full_name),
        //     password: ActiveValue::Set(password),
        //     email: ActiveValue::Set(email),
        //     phone: ActiveValue::Set(phone),
        //     ..Default::default()
        // };

        let res = account::Entity::insert(new_account).exec(&self.db).await.to_local_error()?;
        Ok(res.last_insert_id)
    }

    pub async fn get_product(&self, product_id: i32) -> Result<Value, LocalError> {
        let product: Option<product::Model> = product::Entity::find_by_id(product_id)
            .one(&self.db)
            .await
            .to_local_error()?;

        Ok(json!(product))
    }

    pub async fn get_all_products(&self) -> Result<Value, LocalError> {
        Ok(json!(product::Entity::find()
            .all(&self.db)
            .await
            .to_local_error()?))
    }

    pub async fn add_product(&self, product_json: Value) -> Result<i32, LocalError> {
        let new_product = product::ActiveModel::from_json(product_json).to_local_error()?;
        let res = product::Entity::insert(new_product).exec(&self.db).await.to_local_error()?;
        Ok(res.last_insert_id)
    }

    pub async fn delete_product(&self, product_id: i32) -> Result<(), LocalError> {
        let _ = product::Entity::delete_by_id(product_id)
            .exec(&self.db)
            .await.to_local_error()?;

        Ok(())
    }

    pub async fn purchase(
        &self,
        product_id: i32,
        count: i32,
        user_id: i32,
    ) -> Result<Value, LocalError> {
        let product: Option<product::Model> = product::Entity::find_by_id(product_id)
            .one(&self.db)
            .await
            .to_local_error()?;

        if product.is_none() {
            return Err(LocalError::IdNotFound);
        }

        let product = product.unwrap();
        if !product.is_available(Some(count)) {
            return Err(LocalError::ItemNotAvailable);
        }

        let order = orders::ActiveModel {
            order_id: ActiveValue::NotSet,
            user_id: ActiveValue::Set(user_id),
            product_id: ActiveValue::Set(product.product_id),
            date_time: ActiveValue::Set(Utc::now().naive_utc()),
            total_price: ActiveValue::Set(product.price),
        };

        let res = orders::Entity::insert(order)
            .exec(&self.db)
            .await
            .to_local_error()?;
        let order_id = res.last_insert_id;

        let product_count = product.count;
        let mut product: product::ActiveModel = product.into();
        product.count = ActiveValue::Set(product_count - count);

        let product: Result<product::Model, LocalError> = product.update(&self.db).await.to_local_error();

        match product {
            Ok(product) => Ok(json!(product)),
            Err(e) => {
                let order: Result<Option<orders::Model>, LocalError> =
                    orders::Entity::find_by_id(order_id)
                        .one(&self.db)
                        .await
                        .to_local_error();
                if order.is_err() {
                    let _: Result<Option<orders::Model>, DbErr> =
                        orders::Entity::find_by_id(order_id).one(&self.db).await;
                }
                Err(e)
            }
        }
    }
}
