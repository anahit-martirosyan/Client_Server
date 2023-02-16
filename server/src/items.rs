use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub image: String,
    pub count: i64,
    pub price: f64,
    pub category: String,
}

impl Item {
    pub fn is_available(&self) -> bool {
        return self.count > 0;
    }
}

impl AsRef<Item> for Item {
    fn as_ref(&self) -> &Item {
        &self
    }
}

impl<'a> From<&'a Value> for Item {
    fn from(json: &'a Value) -> Self {
        let json = json.as_object().unwrap();
        Self {
            id: json.get("id").unwrap().as_str().unwrap().to_string(),
            name: json.get("name").unwrap().as_str().unwrap().to_string(),
            image: json.get("image").unwrap().as_str().unwrap().to_string(),
            count: json.get("count").unwrap().as_i64().unwrap(),
            price: json.get("price").unwrap().as_f64().unwrap(),
            category: json.get("category").unwrap().as_str().unwrap().to_string(),
        }
    }
}

impl Into<Value> for Item {
    fn into(self) -> Value {
        let mut obj: Map<String, Value> = Map::new();
        obj.insert("id".to_string(), Value::from(self.id));
        obj.insert("name".to_string(), Value::from(self.name));
        obj.insert("image".to_string(), Value::from(self.image));
        obj.insert("status".to_string(), Value::from(self.count > 0));
        obj.insert("price".to_string(), Value::from(self.price));
        obj.insert("category".to_string(), Value::from(self.category));

        Value::Object(obj)
    }
}
