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
pub use diesel_migrations;

use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use objects::{AttributeRepo, CharacterAccountRepo, ObjectRepo, ScriptRepo, TagRepo};
use structures::account::AccountHandler;

/// All SQL migrations under `migrations/`, embedded at compile time.
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../migrations");

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
    /// scheduled-job CRUD
    pub scripts: ScriptRepo,
}

impl DatabaseHandler {
    /// Connect to the configured database, apply runtime pragmas, and run
    /// any pending migrations.
    ///
    /// Reads `DATABASE_URL`; falls back to [`default_url`] (`data/world.db`
    /// for SQLite or `postgres://muoxi:muoxi@localhost/muoxi` for Postgres).
    /// Panics if the connection cannot be established or if migrations fail.
    pub fn connect() -> Self {
        let mut handle = establish();
        configure(&mut handle).expect("configure() pragmas failed");
        handle
            .run_pending_migrations(MIGRATIONS)
            .expect("run_pending_migrations failed");
        Self {
            handle,
            accounts: AccountHandler,
            objects: ObjectRepo,
            attributes: AttributeRepo,
            tags: TagRepo,
            character_accounts: CharacterAccountRepo,
            scripts: ScriptRepo,
        }
    }
}
