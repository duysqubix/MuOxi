pub mod clients;
pub mod utils;

use bson::{bson, doc, Bson, Document, EncoderResult};
use mongodb::error::Result as MongoResult;
use mongodb::error::{Error, ErrorKind};
use mongodb::options::*;
use mongodb::{Client, Collection, Cursor, Database};
use serde::Serialize;
use utils::{FilterOn, MongoDocument};

#[derive(Debug, Clone)]
pub struct DatabaseHandler {
    pub client: Option<Client>,
    pub db: Option<Database>,
}

impl DatabaseHandler {
    pub fn new(name: String) -> MongoResult<Self> {
        let mut client_options = ClientOptions::parse("mongodb://localhost:27017")?;
        client_options.app_name = Some(name);

        let client = Client::with_options(client_options)?;

        Ok(Self {
            client: Some(client),
            db: None,
        })
    }

    pub fn db_name(&self) -> &str {
        if let Some(db) = &self.db {
            db.name()
        } else {
            "None"
        }
    }

    pub fn set_db<'a>(&mut self, db: &'a str) -> MongoResult<()> {
        if let Some(client) = &self.client {
            self.db = Some(client.database(db));
            println!("DB set to {:?}", self.db_name());
        }
        Ok(())
    }

    pub fn get_db<'a>(&mut self) -> MongoResult<&Database> {
        if let Some(db) = &self.db {
            Ok(db)
        } else {
            let err = Error::from(ErrorKind::OperationError {
                message: "Error finding database".to_string(),
            });
            Err(err)
        }
    }

    pub fn get_collection<'a>(&mut self, collection: &'a str) -> MongoResult<Collection> {
        let db = self.get_db()?;
        let col = db.collection(collection);
        Ok(col)
    }

    pub fn collection_exists(&mut self, col: &Collection) -> MongoResult<bool> {
        let db = self.get_db().unwrap();
        let collection_names = db.list_collection_names(None)?;

        for cname in collection_names {
            if cname.as_str() == col.name() {
                return Ok(true);
            }
        }

        Err(ErrorKind::OperationError {
            message: "collection doesn't exist".to_string(),
        }
        .into())
    }

    ///
    /// Will error if attempting to insert to a collection that doesn't exist
    /// on database
    ///
    pub fn insert_one<T: Serialize + MongoDocument>(
        &mut self,
        object: &T,
        collection: &Collection,
        options: impl Into<Option<InsertOneOptions>>,
    ) -> MongoResult<()> {
        // first check to see if collection exists within database
        self.collection_exists(&collection)?;
        //self.doc_exists(&object, &collection, FilterOn::UID);
        if self
            .doc_exists(object, collection, FilterOn::UID, None)
            .unwrap()
        {
            println!("{}:{} already in database", object.uid(), object.name());
            return Ok(());
        } else {
            println!("It does not exist! Creating new instance :)")
        }

        let ser_obj = self.serialize_obj(&object)?;
        if let bson::Bson::Document(document) = ser_obj {
            collection.insert_one(document, options)?;
        } else {
            println!("Error converting the BSON object into a MongoDB document");
        }
        Ok(())
    }

    pub fn doc_exists<T: Serialize + MongoDocument>(
        &mut self,
        object: &T,
        collection: &Collection,
        filter_on: utils::FilterOn,
        options: impl Into<Option<CountOptions>>,
    ) -> MongoResult<bool> {
        let number = match filter_on {
            FilterOn::NAME => {
                let filter = doc! {"name": object.name()};
                let num_of_documents = collection.count_documents(filter, options)?;
                num_of_documents
            }
            FilterOn::UID => {
                let filter = doc! {"uid": object.uid()};
                let num_of_documents = collection.count_documents(filter, options)?;
                num_of_documents
            }
        };
        if number > 0 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn get_docs<T: Serialize + MongoDocument>(
        &mut self,
        object: &T,
        collection: &Collection,
        filter_on: utils::FilterOn,
        find_options: impl Into<Option<FindOptions>>,
    ) -> MongoResult<Cursor> {
        let cursor = match filter_on {
            FilterOn::NAME => {
                let filter = doc! {"name": object.name()};
                let cursor = collection.find(filter, find_options);
                cursor
            }
            FilterOn::UID => {
                let filter = doc! {"uid" : object.uid()};
                let cursor = collection.find(filter, find_options);
                cursor
            }
        };

        cursor
    }

    pub fn get_doc<T: Serialize + MongoDocument>(
        &mut self,
        object: &T,
        collection: &Collection,
        filter_on: FilterOn,
        find_options: impl Into<Option<FindOneOptions>>,
    ) -> MongoResult<Option<Document>> {
        let document = match filter_on {
            FilterOn::NAME => {
                let filter = doc! {"name": object.name()};
                let document = collection.find_one(filter, find_options);
                document
            }
            FilterOn::UID => {
                let filter = doc! {"uid": object.uid()};
                let document = collection.find_one(filter, find_options);
                document
            }
        };

        document
    }

    ///
    /// Updates default to all instances specified in
    /// filter to all documents in collection
    ///
    pub fn update<T: Serialize + MongoDocument>(
        &mut self,
        object: &T,
        collection: &Collection,
        filter_on: utils::FilterOn,
        find_options: impl Into<Option<FindOptions>>,
        update_options: Option<UpdateOptions>,
    ) -> MongoResult<()> {
        self.collection_exists(&collection)?;

        // take object and extract either uid or name to search on from database
        let cursor = self.get_docs(object, collection, filter_on, find_options)?;
        let updated_document = self.serialize_obj(&object)?;

        for result in cursor {
            match result {
                Ok(document) => {
                    if let Bson::Document(d) = updated_document.clone() {
                        collection.update_one(document, d, None)?;
                    }
                }
                Err(e) => println!("Error occured: {:?}", e),
            }
        }

        Ok(())
    }

    ///
    /// Helper function for more ergonomic approach to serializing an object for mongodb
    /// consumption
    ///
    fn serialize_obj<T: Serialize>(&self, object: &T) -> EncoderResult<bson::Bson> {
        let ser_obj = bson::to_bson(object)?;
        Ok(ser_obj)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
