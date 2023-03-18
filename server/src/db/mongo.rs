use chrono::Utc;
use mongodb::bson::{doc, Document};
use mongodb::{error::Result as MongoResult, Database as MongoDatabase};


pub struct MongoDB {
    pub db: MongoDatabase,
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