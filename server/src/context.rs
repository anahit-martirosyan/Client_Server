use crate::db::DB;
use crate::redis_cache::Cache;

pub struct Context {
    pub db: DB,
    pub cache: Cache,
}
