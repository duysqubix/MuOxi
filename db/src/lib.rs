//! Diesel-powered ORM management library for MuOxi.
//!
//! Default backend is SQLite via the `db-sqlite` feature; Postgres is opt-in
//! via `db-postgres`. The active backend's connection type is exposed as
//! [`Conn`].

pub mod cache;
pub mod cache_structures;
pub mod conn;
pub mod schema;
pub mod structures;
pub mod utils;

pub use conn::{Conn, configure, default_url, establish};

pub use diesel;

/// Main database handler.
pub struct DatabaseHandler {
    /// actual connection to the database (SQLite or Postgres)
    pub handle: Conn,
    /// handle to the Accounts table
    pub accounts: structures::account::AccountHandler,
    /// handle to the Characters table
    pub characters: structures::character::CharacterHandler,
    /// handle to the account_characters join table
    pub account_characters: structures::account_character_link::AccountCharacterHandler,
}

impl DatabaseHandler {
    /// Connect to the configured database and apply runtime PRAGMAs.
    ///
    /// Reads `DATABASE_URL`; falls back to [`default_url`] (`data/world.db`
    /// for SQLite or `postgres://muoxi:muoxi@localhost/muoxi` for Postgres).
    /// Panics if the connection cannot be established.
    pub fn connect() -> Self {
        let mut handle = establish();
        configure(&mut handle).expect("configure() pragmas failed");
        Self {
            handle,
            accounts: structures::account::AccountHandler,
            characters: structures::character::CharacterHandler,
            account_characters:
                structures::account_character_link::AccountCharacterHandler,
        }
    }
}
