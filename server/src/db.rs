use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, Database, DatabaseConnection,
    DbErr, EntityTrait, QueryFilter, Statement,
};
use serde_json::{json, Value};

use crate::entities::{account, orders, product};
use crate::utils::LocalError;

use chrono::Utc;
use mongodb::bson::{doc, Document};
use mongodb::{error::Result as MongoResult, Client, Database as MongoDatabase};

trait ToError<T> {
    fn to_local_error(self, record_type: RecordType) -> Result<T, LocalError>;
}

enum RecordType {
    User,
    Product,
    Order,
}

impl<T> ToError<T> for Result<T, DbErr> {
    fn to_local_error(self, record_type: RecordType) -> Result<T, LocalError> {
        match self {
            Ok(t) => Ok(t),
            Err(DbErr::RecordNotFound(_)) => match record_type {
                RecordType::Product | RecordType::Order => Err(LocalError::IdNotFound),
                RecordType::User => Err(LocalError::WrongUserOrPassword),
            },
            Err(DbErr::Json(_)) | Err(DbErr::Type(_)) => Err(LocalError::WrongParameters),
            Err(_) => Err(LocalError::OperationFailed),
        }
    }
}

pub struct DB {
    pub postgres_db: PostgresDB,
    pub mongo_db: MongoDB,
}

pub struct PostgresDB {
    db: DatabaseConnection,
}

impl PostgresDB {
    pub async fn add_user(&self, user_json: Value) -> Result<i32, LocalError> {
        // let username = user_json.get("username").unwrap().to_string();
        // let full_name = user_json.get("full_name").unwrap().to_string();
        // let password = user_json.get("password").unwrap().to_string();
        // let email = user_json.get("email").unwrap().to_string();
        // let phone = user_json.get("phone").unwrap().to_string();

        let new_account =
            account::ActiveModel::from_json(user_json).to_local_error(RecordType::User)?;
        //     account::ActiveModel {
        //     username: ActiveValue::Set(username),
        //     full_name: ActiveValue::Set(full_name),
        //     password: ActiveValue::Set(password),
        //     email: ActiveValue::Set(email),
        //     phone: ActiveValue::Set(phone),
        //     ..Default::default()
        // };

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

    pub async fn get_product(&self, product_id: i32) -> Result<Value, LocalError> {
        let product: Option<product::Model> = product::Entity::find_by_id(product_id)
            .one(&self.db)
            .await
            .to_local_error(RecordType::Product)?;

        Ok(json!(product))
    }

    pub async fn get_all_products(&self) -> Result<Value, LocalError> {
        Ok(json!(product::Entity::find()
            .all(&self.db)
            .await
            .to_local_error(RecordType::Product)?))
    }

    pub async fn add_product(&self, product_json: Value) -> Result<i32, LocalError> {
        let new_product =
            product::ActiveModel::from_json(product_json).to_local_error(RecordType::Product)?;
        let res = product::Entity::insert(new_product)
            .exec(&self.db)
            .await
            .to_local_error(RecordType::Product)?;
        Ok(res.last_insert_id)
    }

    pub async fn delete_product(&self, product_id: i32) -> Result<(), LocalError> {
        let _ = product::Entity::delete_by_id(product_id)
            .exec(&self.db)
            .await
            .to_local_error(RecordType::Product)?;

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
            .to_local_error(RecordType::Product)?;

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
            .to_local_error(RecordType::Order)?;
        let order_id = res.last_insert_id;

        let product_count = product.count;
        let mut product: product::ActiveModel = product.into();
        product.count = ActiveValue::Set(product_count - count);

        let product: Result<product::Model, LocalError> = product
            .update(&self.db)
            .await
            .to_local_error(RecordType::Product);

        match product {
            Ok(product) => Ok(json!(product)),
            Err(e) => {
                let order: Result<Option<orders::Model>, LocalError> =
                    orders::Entity::find_by_id(order_id)
                        .one(&self.db)
                        .await
                        .to_local_error(RecordType::Order);
                if order.is_err() {
                    let _: Result<Option<orders::Model>, DbErr> =
                        orders::Entity::find_by_id(order_id).one(&self.db).await;
                }
                Err(e)
            }
        }
    }
}

pub struct MongoDB {
    db: MongoDatabase,
}

impl MongoDB {
    pub async fn add_user(&self, user_id: i32) -> MongoResult<()> {
        let collection = self.db.collection::<Document>("records");

        let record = doc! {"user_id": user_id, "account_created": Utc::now().to_string()};

        collection
            .insert_one(record, None)
            .await
            .and_then(|_| Ok(()))
    }

    pub async fn record_logged_in(&self, user_id: i32) -> Result<(), mongodb::error::Error> {
        let filter = doc! { "user_id": user_id };
        let collection = self.db.collection::<Document>("records");
        let update = doc! {"$set": {"last_logged_in": Utc::now().to_string()}};
        let res = collection
            .update_one(filter.clone(), update.clone(), None)
            .await?;
        if res.matched_count == 0 {
            let _ = self.add_user(user_id);
            let _ = collection.update_one(filter, update, None).await?;
        }

        Ok(())
    }
}

impl DB {
    #[tokio::main]
    pub async fn init(
        postgres_uri: &str,
        postgres_name: &str,
        mongo_uri: &str,
        mongo_name: &str,
    ) -> Option<DB> {
        let postgres = DB::init_postgres(postgres_uri, postgres_name).await;
        let mongo = DB::init_mongo(mongo_uri, mongo_name).await;

        if postgres.is_err() || mongo.is_err() {
            return None;
        }

        Some(DB {
            postgres_db: PostgresDB {
                db: postgres.unwrap(),
            },
            mongo_db: MongoDB { db: mongo.unwrap() },
        })
    }

    async fn init_postgres(uri: &str, name: &str) -> Result<DatabaseConnection, DbErr> {
        let statement = format!("SELECT \'CREATE DATABASE {}\' WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = \'{}\')", name, name);
        println!("{}", statement);
        let db = Database::connect(uri).await?;
        db.execute(Statement::from_string(db.get_database_backend(), statement))
            .await?;

        let uri = format!("{}/{}", uri, name);
        let db_con = Database::connect(&uri).await?;

        Ok(db_con)
    }

    async fn init_mongo(uri: &str, name: &str) -> MongoResult<MongoDatabase> {
        let client = Client::with_uri_str(uri).await?;

        Ok(client.database(name))
    }
}
