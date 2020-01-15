//!
//! Diesel powered ORM management library for MuOxi
//! uses postgres
//!

#[macro_use]
extern crate diesel;

// pub mod accounts;
pub mod cache;
pub mod cache_structures;
pub mod schema;
pub mod utils;

use diesel::pg::PgConnection;
use diesel::prelude::*;

/// Main database handler struct
pub struct DatabaseHandler {
    /// acutal connection to postgres database
    pub handle: PgConnection,
    // / handle to the clients table
    // pub clients: clients::ClientHandler,
}

impl DatabaseHandler {
    /// creates a new instance of the handler and defaults to
    /// postgres url: `postgres://muoxi:muoxi@localhost/muoxi`
    /// When setting up postgresql create user name `muoxi` and password `muoxi`
    /// and/or replace the internal url variable in this function with an appropriate
    /// and valid url. It will panic if it can't connect to a database.
    pub fn connect() -> Self {
        let url = "postgres://muoxi:muoxi@localhost/muoxi".to_string();
        let conn = PgConnection::establish(&url).expect("Couldn't create handle to database");
        Self {
            handle: conn,
            // clients: clients::ClientHandler {},
        }
    }
}
