//!
//! Rust implmentation of MongoDB Client. One must have already started a MongoDB server, otherwise
//! when attempting to create a handler to the database, it will panic.
//!
//! ### Basic Usage
//! ```rust, ignore
//! let mut mongo = DatabaseHandler::new("Caller".to_string()).unwrap(); // will panic if no mongodb server is running.
//! mongo.set_db("test").unwrap(); // sets internal database
//! ```
//!

pub mod templates;
pub mod utils;

use bson::{bson, doc, Bson, Document, EncoderResult};
use mongodb::error::Result as MongoResult;
use mongodb::error::{Error, ErrorKind};
use mongodb::options::*;
use mongodb::{Client, Collection, Cursor, Database};
use serde::Serialize;
use utils::{FilterOn, MongoDocument};

/// Wrapper to MongoDB running in background
#[derive(Debug, Clone)]
pub struct DatabaseHandler {
    /// Holds the actual client handler to MongoDB
    pub client: Option<Client>,

    /// Database handler within MongoDB
    /// set using `DatabaseHandler::set_db()`
    pub db: Option<Database>,
}

impl DatabaseHandler {
    /// Create a new handler to MongoDB
    pub fn new(name: String) -> MongoResult<Self> {
        let mut client_options = ClientOptions::parse("mongodb://localhost:27017")?;
        client_options.app_name = Some(name);

        let client = Client::with_options(client_options)?;

        Ok(Self {
            client: Some(client),
            db: None,
        })
    }

    /// retrieve current database name
    pub fn db_name(&self) -> &str {
        if let Some(db) = &self.db {
            db.name()
        } else {
            "None"
        }
    }

    /// set database within client, all actions
    /// will be targeted to supplied database name.
    pub fn set_db<'a>(&mut self, db: &'a str) -> MongoResult<()> {
        if let Some(client) = &self.client {
            self.db = Some(client.database(db));
            println!("DB set to {:?}", self.db_name());
        }
        Ok(())
    }

    /// Returns reference to Database Handler, if db is not set
    /// it will return Error;
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

    /// Returns Collection within current database
    pub fn get_collection<'a>(&mut self, collection: &'a str) -> MongoResult<Collection> {
        let db = self.get_db()?;
        let col = db.collection(collection);
        Ok(col)
    }

    /// Checks if collection exists within database
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

    /// Attempts to serialize a rust object into a valid mongo document.
    /// Object must have `MongoDocument + Serialize` traits in order to be
    /// considered valid
    ///
    /// Will error if attempting to insert to a collection that doesn't exist
    /// on database
    ///
    /// ### Basic Usage
    /// ```rust, ignore
    /// type UID = u64;
    /// struct Crab{
    ///     uid: UID,
    ///     name: String,
    /// }
    ///
    /// impl MongoDocument for Crab{
    ///     fn name(&self) -> String{
    ///         self.name.clone()
    ///     }
    ///     
    ///     fn uid(&self) -> UID{
    ///         self.uid
    ///     }
    /// }
    ///
    /// let crab = Crab{
    ///     uid: db::utils::gen_uid(),
    ///     name: "Crab".to_string();
    /// }
    ///
    /// let mut mongo = DatabaseHandler::new("Caller".to_string()).unwrap();
    ///
    /// mongo.set_db("entities").unwrap();
    ///
    /// let collection = mongo.get_collection("mobs").expect("Couldn't find collection");
    ///
    /// // serialized and inserted to MongoDB with Schema
    /// // entities -> mobs -> crab_bson
    /// mongo.insert_one(&crab, &collection, None).unwrap();
    /// ```
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

    /// Check to see if a particular document exists within data base
    /// and supplied collection. Filters based on either *UID* which will __always__ be
    /// unique, or *name* which could have multiple instances in collection.
    ///
    /// ```rust, ignore
    /// let crab = Crab::new(); // valid mongo document serializable struct
    ///
    /// let col = mongo.get_collection("mobs").unwrap();
    ///
    /// assert_eq!(true, doc_exists(&crab, &col, FilterOn::UID, None))
    /// ```
    ///
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

    /// Attempts to get a Database Cursor to all
    /// documents specified by Object T
    ///
    /// ```rust, ignore
    /// let cursor = get_docs(&crab, &mobs_collection, FilterOn::NAME, None).unwrap();
    ///
    /// for result in cursor{
    ///     assert_eq!(bson::Document, result);
    /// }
    /// ```
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

    /// Attempts to return a single Document within Collection
    ///
    /// ```rust, ignore
    /// let document = get_doc(&crab, &mob_collection, None).unwrap();
    ///
    /// assert_eq!(bson::Document, document);
    /// ```
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

    /// Updates documents in collection based on FilterOn
    /// __Updates ALL instances__ based on `FilterOn` and `FindOptions`
    ///
    /// ```rust, ignore
    /// let crab = Crab{
    ///     uid: db::utils::gen_uid(),
    ///     name: "little crab",
    /// }
    ///
    /// let crab_doc = mongo.get_doc(&crab, &mob_collection, None, None)?;
    /// assert_neq!(crab, db::utils::bson_to_object(crab_doc)?);
    ///
    /// mongo.update(
    ///     &crab,
    ///     &mob_collection,
    ///     db::utils::FilterOn::UID,
    ///     None,
    ///     None,
    /// )
    ///
    /// let crab_doc = mongo.get_doc(&crab, &mob_collection, None, None)?;
    /// assert_eq!(crab, bson_to_object(crab_doc)?);
    /// ```
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
