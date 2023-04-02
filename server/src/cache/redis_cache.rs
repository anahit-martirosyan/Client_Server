use redis::{Client, Commands, RedisResult};
use serde_json::{json, Map, Value};

pub struct Cache {
    pub redis_client: Client,
}

fn get_product_key(id: i32) -> String {
    format!("product_{}", id)
}

impl Cache {
    pub fn init(redis_uri: String) -> RedisResult<Cache> {
        let client = Client::open(redis_uri.as_str())?;

        Ok(Cache {
            redis_client: client,
        })
    }


    fn add_product_inner(&self, product_id: i32, product: &Map<String, Value>) -> Result<(), ()> {
        let conn = self.redis_client.get_connection();
        if conn.is_err() {
            return Err(());
        }

        let mut conn = conn.unwrap();

        let product_str = serde_json::to_string(&product).unwrap();

        let res: RedisResult<()> = conn.set(get_product_key(product_id), product_str);

        if res.is_err() {
            Err(())
        } else {
            Ok(())
        }
    }

    pub fn get_product(&self, id: i32) -> Option<Value> {
        let conn = self.redis_client.get_connection();
        if conn.is_err() {
            return None;
        }

        let mut conn = conn.unwrap();
        let product = conn.get(get_product_key(id));
        if product.is_err() {
            return None;
        }

        let product: String = product.unwrap();

        Some(json!(product))
    }

    pub fn add_product(&self, id: i32, mut json: Value) -> Result<(), ()> {
        let product = json.as_object_mut().ok_or(())?;
        product.insert(String::from("id"), json!(id));

        self.add_product_inner(id, product)
    }

    pub fn update_product(&self, id: i32, updates: Value) -> Result<(), ()> {
        let conn = self.redis_client.get_connection();
        if conn.is_err() {
            return Err(());
        }
        let mut conn = conn.unwrap();

        let product = conn.get(get_product_key(id));
        if product.is_err() {
            return Err(());
        }

        let product: Option<String> = product.unwrap();

        if product.is_none() {
            return Ok(());
        }

        let product_json: serde_json::Result<Value> = serde_json::from_str(&product.unwrap());
        if product_json.is_err() {
            return Err(());
        }
        let mut product = product_json.unwrap();
        let product = product.as_object_mut().ok_or(())?;

        let updates: Map<String, Value> = updates.as_object().unwrap().clone();
        for (key, val) in updates.iter() {
            product.insert(key.to_string(), val.clone()).ok_or(())?;
        }

        self.add_product_inner(id, product)
    }

    pub fn delete_product(&self, id: i32) -> Result<(), ()> {
        let conn = self.redis_client.get_connection();
        if conn.is_err() {
            return Err(());
        }

        let mut conn = conn.unwrap();

        let res: RedisResult<()>  = conn.del(get_product_key(id));

        if res.is_err() {
            Err(())
        } else {
            Ok(())
        }
    }
}