use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection,
    DbErr, EntityTrait, QueryFilter, entity::*
};
use serde_json::{json, Map, Value};

use crate::entities::{account, orders, product};
use crate::utils::{LocalError, round};

use chrono::Utc;
use sea_orm::prelude::Decimal;


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
            Err(DbErr::RecordNotFound(e)) =>{
                println!("{}", e);
                match record_type {
                    RecordType::Product | RecordType::Order => Err(LocalError::IdNotFound),
                    RecordType::User => Err(LocalError::WrongUserOrPassword),
                }
            },
            Err(DbErr::Json(e)) | Err(DbErr::Type(e)) => {
                println!("{}", e);
                Err(LocalError::WrongParameters)
            },
            Err(e) => {
                println!("{}", e);
                Err(LocalError::OperationFailed)
            },
        }
    }
}



pub struct PostgresDB {
    pub db: DatabaseConnection,
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
        let mut products =
        product::Entity::find()
            .all(&self.db)
            .await
            .to_local_error(RecordType::Product)?;

        for mut prod in products.iter_mut() {
            prod.status = prod.count > 0;
        }

        Ok(json!(products))
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

    pub async fn update_product(&self, product_id: i32, updates: Value) -> Result<Value, LocalError> {
        let product: Option<product::Model> = product::Entity::find_by_id(product_id)
            .one(&self.db)
            .await
            .to_local_error(RecordType::Product)?;

        if product.is_none() {
            return Err(LocalError::OperationFailed);
        }

        let mut product: product::ActiveModel = product.unwrap().into();

        let updates: Map<String, Value> = updates.as_object().unwrap().clone();
        for (key, val) in updates.iter() {
            match key.as_str() {
                "name" => {
                    let name = val.as_str();
                    if name.is_none() {
                        return Err(LocalError::WrongParameters);
                    }
                    product.name = Set(name.unwrap().to_string());
                },
                "image" => {
                    let image = val.as_str();
                    if image.is_none() {
                        return Err(LocalError::WrongParameters);
                    }
                    product.image = Set(Some(image.unwrap().to_string()));
                },
                "count" => {
                    let count = val.as_i64();
                    if count.is_none() {
                        return Err(LocalError::WrongParameters);
                    }
                    product.count = Set(count.unwrap() as i32);
                },
                "price" => {
                    let price = val.as_f64();
                    if price.is_none() {
                        return Err(LocalError::WrongParameters);
                    }
                    let price = Decimal::from_str_exact( &round(price.unwrap(), 2).to_string());
                    if price.is_err() {
                        return Err(LocalError::WrongParameters);
                    }
                    product.price = Set(price.unwrap());
                },
                "category" => {
                    let category = val.as_str();
                    if category.is_none() {
                        return Err(LocalError::WrongParameters);
                    }
                    product.category = Set(category.unwrap().to_string());
                },
                _ => {}
            }
        }

        let product: product::Model = product.update(&self.db).await.to_local_error(RecordType::Product)?;

        Ok(json!(product))
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



