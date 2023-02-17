use crate::items::Item;
use serde_json::Value;
use std::borrow::BorrowMut;
use std::collections::HashMap;

use crate::utils;

static PRODUCTS_FILE: &str = "./data/products.json";

pub fn read_items() -> Result<HashMap<String, Item>, &'static str> {
    let json = std::fs::read_to_string(PRODUCTS_FILE);
    if json.is_err() {
        return Err(utils::OPERATION_FAILED);
    }
    let products = serde_json::from_str::<Value>(&json.unwrap());
    if products.is_err() {
        return Err(utils::OPERATION_FAILED);
    }
    let mut items: HashMap<String, Item> = HashMap::new();
    for item in products.unwrap().as_array().unwrap() {
        let item: Item = item.into();
        items.insert(item.id.to_string(), item);
    }

    Ok(items)
}

pub fn purchase_item(id: &str, count: i64) -> Result<Item, &'static str> {
    let mut items = read_items()?;

    let item = items.get_mut(id).ok_or(utils::ID_NOT_FOUND)?;

    if !item.is_available() {
        return Err(utils::ITEM_NOT_AVAILABLE);
    }
    item.count -= count;

    let it = (*item).clone();

    match update_items(&items) {
        Err(e) => Err(e),
        Ok(_) => Ok(it),
    }
}

pub fn get_new_id() -> Result<String, &'static str> {
    let items = read_items()?;
    let last_id = items
        .keys()
        .max()
        .and_then(|id| id.parse::<i64>().ok())
        .ok_or(utils::OPERATION_FAILED)?;

    return Ok((last_id + 1).to_string());
}

pub fn get_item(id: &str) -> Result<Item, &'static str> {
    let items = read_items()?;

    items.get(id).cloned().ok_or(utils::ID_NOT_FOUND)
}

pub fn add_item(item: Item) -> Result<(), &'static str> {
    let mut items = read_items()?;
    let items = items.borrow_mut();
    items.insert(item.as_ref().id.clone(), item);

    update_items(items)
}

pub fn delete_item(id: &str) -> Result<(), &'static str> {
    let mut items = read_items()?;
    let items = items.borrow_mut();

    if items.remove(id).is_none() {
        return Err(utils::ID_NOT_FOUND);
    }

    update_items(items)
}

pub fn update_items(items: &HashMap<String, Item>) -> Result<(), &'static str> {
    let items_vec: Vec<&Item> = items.values().collect();
    if std::fs::write(
        PRODUCTS_FILE,
        serde_json::to_string_pretty(&items_vec).unwrap_or_default(),
    )
    .is_err()
    {
        return Err(utils::OPERATION_FAILED);
    }

    Ok(())
}
