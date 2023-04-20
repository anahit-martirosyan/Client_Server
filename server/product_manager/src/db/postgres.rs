use sea_orm::{entity::*, ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait};
use serde_json::{json, Map, Value};

use crate::entities::product;
use common::db_utils::{RecordType, ToError};
use common::utils::{round, LocalError};

use sea_orm::prelude::Decimal;

pub struct PostgresDB {
    pub db: DatabaseConnection,
}

impl PostgresDB {
    pub async fn get_product(&self, product_id: i32) -> Result<Value, LocalError> {
        let product: Option<product::Model> = product::Entity::find_by_id(product_id)
            .one(&self.db)
            .await
            .to_local_error(RecordType::Product)?;

        Ok(json!(product))
    }

    pub async fn get_all_products(&self) -> Result<Value, LocalError> {
        let mut products = product::Entity::find()
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

    pub async fn update_product(
        &self,
        product_id: i32,
        updates: Value,
    ) -> Result<Value, LocalError> {
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
                }
                "image" => {
                    let image = val.as_str();
                    if image.is_none() {
                        return Err(LocalError::WrongParameters);
                    }
                    product.image = Set(Some(image.unwrap().to_string()));
                }
                "count" => {
                    let count = val.as_i64();
                    if count.is_none() {
                        return Err(LocalError::WrongParameters);
                    }
                    product.count = Set(count.unwrap() as i32);
                }
                "price" => {
                    let price = val.as_f64();
                    if price.is_none() {
                        return Err(LocalError::WrongParameters);
                    }
                    let price = Decimal::from_str_exact(&round(price.unwrap(), 2).to_string());
                    if price.is_err() {
                        return Err(LocalError::WrongParameters);
                    }
                    product.price = Set(price.unwrap());
                }
                "category" => {
                    let category = val.as_str();
                    if category.is_none() {
                        return Err(LocalError::WrongParameters);
                    }
                    product.category = Set(category.unwrap().to_string());
                }
                _ => {}
            }
        }

        let product: product::Model = product
            .update(&self.db)
            .await
            .to_local_error(RecordType::Product)?;

        Ok(json!(product))
    }

    pub async fn delete_product(&self, product_id: i32) -> Result<(), LocalError> {
        let _ = product::Entity::delete_by_id(product_id)
            .exec(&self.db)
            .await
            .to_local_error(RecordType::Product)?;

        Ok(())
    }

    pub async fn purchase(&self, product_id: i32, count: i32) -> Result<Value, LocalError> {
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

        let product_count = product.count;
        let mut product: product::ActiveModel = product.into();
        product.count = ActiveValue::Set(product_count - count);

        let product: Result<product::Model, LocalError> = product
            .update(&self.db)
            .await
            .to_local_error(RecordType::Product);

        match product {
            Ok(product) => Ok(json!(product)),
            Err(e) => Err(e),
        }
    }
}
