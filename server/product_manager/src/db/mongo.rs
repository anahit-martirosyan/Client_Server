use chrono::Utc;
use mongodb::bson::{doc, Document};
use mongodb::{error::Result as MongoResult, Database as MongoDatabase};

pub struct MongoDB {
    pub db: MongoDatabase,
}

impl MongoDB {
    pub async fn add_product(&self, product_id: i32) -> MongoResult<()> {
        let collection = self.db.collection::<Document>("products_stats");

        let record = doc! {
            "product_id": product_id,
            "viewed": [],
            "purchased": [],
        };

        collection
            .insert_one(record, None)
            .await
            .and_then(|_| Ok(()))
    }

    pub async fn record_product_viewed(
        &self,
        product_id: i32,
    ) -> Result<(), mongodb::error::Error> {
        let update = doc! {"$push": {"viewed": Utc::now().to_string()}};

        self.update_product_stats(product_id, update).await
    }

    pub async fn record_product_purchased(
        &self,
        product_id: i32,
    ) -> Result<(), mongodb::error::Error> {
        let update = doc! {"$push": {"purchased.{}": Utc::now().to_string()}};

        self.update_product_stats(product_id, update).await
    }

    async fn update_product_stats(
        &self,
        product_id: i32,
        updates: Document,
    ) -> Result<(), mongodb::error::Error> {
        let filter = doc! { "product_id": product_id };
        let collection = self.db.collection::<Document>("product_stats");

        let res = collection
            .update_one(filter.clone(), updates.clone(), None)
            .await?;
        if res.matched_count == 0 {
            let _ = self.add_product(product_id);
            let _ = collection.update_one(filter, updates, None).await?;
        }

        Ok(())
    }
}
