use async_trait::async_trait;
use mongodb::bson::{doc, Document};
use mongodb::{options::ClientOptions, Client};
use serde::{de::DeserializeOwned, Serialize};

use super::handler::KvStoreConnection;

#[derive(Debug, Clone)]
pub struct MongoDbIndex {
    pub db_name: String,
    pub coll_name: String,
}

#[derive(Debug, Clone)]
pub struct MongoDbConn {
    pub client: Client,
    pub index: MongoDbIndex,
}

#[async_trait]
impl KvStoreConnection for MongoDbConn {
    type ConnectionResult = MongoDbConn;
    type SetDataResult = Result<(), mongodb::error::Error>;
    type GetDataResult<T> = Result<Option<T>, mongodb::error::Error>;

    async fn init(url: &str) -> Self::ConnectionResult {
        let client_options = match ClientOptions::parse(url).await {
            Ok(client_options) => client_options,
            Err(e) => panic!("Failed to connect to MongoDB instance with error: {e}"),
        };

        let client = match Client::with_options(client_options) {
            Ok(client) => client,
            Err(e) => panic!("Failed to connect to MongoDB instance with error: {e}"),
        };

        let index = MongoDbIndex {
            db_name: String::from("default"),
            coll_name: String::from("default"),
        };

        MongoDbConn { client, index }
    }

    async fn set_data<T: Serialize + std::marker::Send>(
        &mut self,
        key: &str,
        value: T,
    ) -> Self::SetDataResult {
        let collection = self
            .client
            .database(&self.index.db_name)
            .collection::<Document>(&self.index.coll_name);

        let document = match mongodb::bson::to_document(&value) {
            Ok(document) => document,
            Err(e) => panic!("Failed to serialize data with error: {e}"),
        };

        println!("Document: {:?}", document);

        let filter = doc! { "_id": key };
        match collection
            .replace_one(
                filter,
                document.clone(),
                mongodb::options::ReplaceOptions::builder()
                    .upsert(true)
                    .build(),
            )
            .await {
            Ok(_) => (),
            Err(e) => panic!("Failed to set data with error: {e}"),
            };

        Ok(())
    }

    async fn get_data<T: DeserializeOwned>(&mut self, key: &str) -> Self::GetDataResult<T> {
        let collection = self
            .client
            .database(&self.index.db_name)
            .collection::<Document>(&self.index.coll_name); // Change to your actual collection name

        let filter = doc! { "_id": key };
        let result = collection.find_one(filter, None).await?;

        if let Some(document) = result {
            let deserialized: T = mongodb::bson::from_document(document)?;
            return Ok(Some(deserialized));
        }
        
        Ok(None)
    }
}
