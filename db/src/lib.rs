//! Diesel-powered ORM management library for MuOxi.
//!
//! Default backend is SQLite via the `db-sqlite` feature; Postgres is opt-in
//! via `db-postgres`. The active backend's connection type is exposed as
//! [`Conn`].

pub mod cache;
pub mod cache_structures;
pub mod conn;
pub mod objects;
pub mod schema;
pub mod structures;
pub mod utils;

pub use conn::{Conn, configure, default_url, establish};

pub use diesel;

use objects::{AttributeRepo, CharacterAccountRepo, ObjectRepo, TagRepo};
use structures::account::AccountHandler;

/// Top-level database facade. Holds an open connection plus all repository
/// helpers. Construct with [`DatabaseHandler::connect`].
pub struct DatabaseHandler {
    /// active database connection
    pub handle: Conn,
    /// account-table CRUD
    pub accounts: AccountHandler,
    /// generic object CRUD
    pub objects: ObjectRepo,
    /// per-object attribute CRUD (JSON-text values)
    pub attributes: AttributeRepo,
    /// per-object tag CRUD
    pub tags: TagRepo,
    /// character⇄account link CRUD
    pub character_accounts: CharacterAccountRepo,
}

impl DatabaseHandler {
    /// Connect to the configured database and apply runtime pragmas.
    ///
    /// Reads `DATABASE_URL`; falls back to [`default_url`] (`data/world.db`
    /// for SQLite or `postgres://muoxi:muoxi@localhost/muoxi` for Postgres).
    /// Panics if the connection cannot be established.
    pub fn connect() -> Self {
        let mut handle = establish();
        configure(&mut handle).expect("configure() pragmas failed");
        Self {
            handle,
            accounts: AccountHandler,
            objects: ObjectRepo,
            attributes: AttributeRepo,
            tags: TagRepo,
            character_accounts: CharacterAccountRepo,
        }
    }
}
