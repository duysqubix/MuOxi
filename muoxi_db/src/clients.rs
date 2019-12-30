//!
//! Holds functions and data structures that relate to connected clients
//! from different comm protocols, stores them on mongodb...
//!

use crate::utils::MongoDocument;
use bson::oid::ObjectId;
use muoxi_states as states;
use serde::{Deserialize, Serialize};
///
/// Struct that holds information about connected client
///
#[derive(Debug, Serialize, Deserialize)]
pub struct ClientDB {
    pub uid: i64,
    pub name: String,
    pub ip: String,
    pub port: i32,
    pub state: states::ConnStates,
    pub ncharacters: i32,
}

impl MongoDocument for ClientDB {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn uid(&self) -> i64 {
        self.uid
    }
}

///
/// Character struct holding information about a mob ingame
///
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Character {
    pub uid: i64,
    pub name: String,
    pub class: String,
    pub gold: i32,
}

impl MongoDocument for Character {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn uid(&self) -> i64 {
        self.uid
    }
}
