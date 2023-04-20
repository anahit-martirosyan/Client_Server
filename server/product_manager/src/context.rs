use crate::db::DB;
use crate::redis_cache::Cache;

pub struct Context {
    pub db: DB,
    pub cache: Cache,
    pub user_manager: UserManagerContext,
    pub order_manager: OrderManagerContext,
}

pub struct UserManagerContext {
    pub uri: String,
    pub product_viewed_endpoint: String,
    pub product_purchased_endpoint: String,
}

pub struct OrderManagerContext {
    pub uri: String,
    pub add_order_endpoint: String,
}