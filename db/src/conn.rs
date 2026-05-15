#![deny(missing_docs)]

//! Backend-selectable connection type.
//!
//! Exactly one of the `db-sqlite` / `db-postgres` features must be enabled;
//! a `compile_error!` is emitted otherwise. SQLite is the default backend.

#[cfg(all(feature = "db-sqlite", feature = "db-postgres"))]
compile_error!(
    "db crate: features `db-sqlite` and `db-postgres` are mutually exclusive. \
     Pick one with --no-default-features --features db-postgres."
);

#[cfg(not(any(feature = "db-sqlite", feature = "db-postgres")))]
compile_error!(
    "db crate: enable exactly one of `db-sqlite` (default) or `db-postgres`."
);

use diesel::prelude::*;
use std::env;

#[cfg(feature = "db-sqlite")]
/// The active connection type for query helpers.
pub type Conn = diesel::sqlite::SqliteConnection;

#[cfg(feature = "db-postgres")]
/// The active connection type for query helpers.
pub type Conn = diesel::pg::PgConnection;

/// Default database URL used when `DATABASE_URL` is not set.
pub fn default_url() -> &'static str {
    #[cfg(feature = "db-sqlite")]
    {
        "data/world.db"
    }
    #[cfg(feature = "db-postgres")]
    {
        "postgres://muoxi:muoxi@localhost/muoxi"
    }
}

/// Open a new connection. Reads `DATABASE_URL` env var, falls back to
/// `default_url()`. Panics on failure.
pub fn establish() -> Conn {
    let url = env::var("DATABASE_URL").unwrap_or_else(|_| default_url().to_string());
    Conn::establish(&url).unwrap_or_else(|e| panic!("db::establish failed for {}: {}", url, e))
}

/// Apply runtime PRAGMAs that the SQLite backend needs. No-op on Postgres.
#[cfg(feature = "db-sqlite")]
pub fn configure(conn: &mut Conn) -> QueryResult<()> {
    diesel::sql_query("PRAGMA journal_mode = WAL").execute(conn)?;
    diesel::sql_query("PRAGMA synchronous = NORMAL").execute(conn)?;
    diesel::sql_query("PRAGMA foreign_keys = ON").execute(conn)?;
    diesel::sql_query("PRAGMA busy_timeout = 5000").execute(conn)?;
    Ok(())
}

/// Apply runtime PRAGMAs that the SQLite backend needs. No-op on Postgres.
#[cfg(feature = "db-postgres")]
pub fn configure(_conn: &mut Conn) -> QueryResult<()> {
    Ok(())
}
