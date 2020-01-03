//!
//! Defines the different structures that are serializable to MongoDB
//! All structures that will have a place in MongoDB need to be defined here.
//!
//! *Note: bson::Document does not support unsigned integer types, therefore the
//! `#[serde(with = "bson::compat::u2f")]` must be supplied for each `uid` field,
//! this converts the uid field to a float.*
//!

use crate::utils::MongoDocument;
use serde::{Deserialize, Serialize};
use states;

pub type UID = u64;

///
/// Struct that holds information about connected clients
///
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientDB {
    /// Unique 8 byte id
    #[serde(with = "bson::compat::u2f")]
    pub uid: UID,

    /// name for connected clients
    pub name: String,

    /// ip address where client is coming from
    pub ip: String,

    /// port client is connected on
    pub port: i32,

    /// current connection state
    pub state: states::ConnStates,
}

impl MongoDocument for ClientDB {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn uid(&self) -> UID {
        self.uid
    }
}

///
/// This WILL change, as this struct is just for testing purposes.
/// Character struct holding information about a mob ingame
///
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Character {
    #[serde(with = "bson::compat::u2f")]
    pub uid: UID,
    pub name: String,
    pub class: String,
    pub gold: i32,
}

impl MongoDocument for Character {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn uid(&self) -> UID {
        self.uid
    }
}
