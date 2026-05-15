//! Diesel-powered ORM management library for MuOxi (Postgres backend).

pub mod cache;
pub mod cache_structures;
pub mod schema;
pub mod structures;
pub mod utils;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::env;

/// Main database handler.
pub struct DatabaseHandler {
    /// actual connection to the postgres database
    pub handle: PgConnection,
    /// handle to the Accounts table
    pub accounts: structures::account::AccountHandler,
    /// handle to the Characters table
    pub characters: structures::character::CharacterHandler,
}

impl DatabaseHandler {
    /// Connect to the Postgres database.
    ///
    /// Reads `DATABASE_URL`, defaults to
    /// `postgres://muoxi:muoxi@localhost/muoxi`. Panics if the
    /// connection cannot be established.
    pub fn connect() -> Self {
        let url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://muoxi:muoxi@localhost/muoxi".to_string());
        let conn = PgConnection::establish(&url).expect("Couldn't create handle to database");
        Self {
            handle: conn,
            accounts: structures::account::AccountHandler,
            characters: structures::character::CharacterHandler,
        }
    }
}
