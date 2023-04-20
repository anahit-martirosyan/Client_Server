use chrono::Utc;
use mongodb::bson::{doc, Document};
use mongodb::{error::Result as MongoResult, Database as MongoDatabase};

pub struct MongoDB {
    pub db: MongoDatabase,
}

impl MongoDB {
    pub async fn add_user(&self, user_id: i32) -> MongoResult<()> {
        let collection = self.db.collection::<Document>("user_stats");

        let record = doc! {
            "user_id": user_id,
            "account_created": Utc::now().to_string(),
            "last_logged_in": Utc::now().to_string(),
            "products_viewed": {},
            "products_purchased": {},
        };

        collection
            .insert_one(record, None)
            .await
            .and_then(|_| Ok(()))
    }

    pub async fn record_logged_in(&self, user_id: i32) -> Result<(), mongodb::error::Error> {
        let update = doc! {"$set": {"last_logged_in": Utc::now().to_string()}};

        self.update_user_record(user_id, update).await
    }

    pub async fn record_product_viewed(
        &self,
        user_id: i32,
        product_id: i32,
    ) -> Result<(), mongodb::error::Error> {
        let update =
            doc! {"$set": {format!("products_viewed.{}", product_id): Utc::now().to_string()}};

        self.update_user_record(user_id, update).await
    }

    pub async fn record_product_purchased(
        &self,
        user_id: i32,
        product_id: i32,
    ) -> Result<(), mongodb::error::Error> {
        let update =
            doc! {"$set": {format!("products_purchased.{}", product_id): Utc::now().to_string()}};

        self.update_user_record(user_id, update).await
    }

    async fn update_user_record(
        &self,
        user_id: i32,
        updates: Document,
    ) -> Result<(), mongodb::error::Error> {
        let filter = doc! { "user_id": user_id };
        let collection = self.db.collection::<Document>("user_stats");

        let res = collection
            .update_one(filter.clone(), updates.clone(), None)
            .await?;
        if res.matched_count == 0 {
            let _ = self.add_user(user_id);
            let _ = collection.update_one(filter, updates, None).await?;
        }

        Ok(())
    }
}
