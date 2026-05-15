#![deny(missing_docs)]

//! Wrapper around the Redis server to easily manipulate cached data.
//!
//! ```ignore
//! use db::cache::Cache;
//! use redis::Commands;
//!
//! let mut conn = Cache::new().expect("Can't connect to redis server");
//! let _: () = conn.set("my_key", 42).unwrap();
//! let value: Option<usize> = conn.get("my_key").unwrap();
//! ```

use redis::{Connection, RedisResult};
use std::env;

/// Factory for `redis::Connection` handles. Reads `REDIS_SERVER` env var,
/// defaults to `redis://127.0.0.1`.
pub struct Cache;

impl Cache {
    /// Open a connection using the supplied URI.
    pub fn new_with_uri(uri: &str) -> RedisResult<Connection> {
        let client = redis::Client::open(uri)?;
        client.get_connection()
    }

    /// Open a connection using `REDIS_SERVER` env var (default `redis://127.0.0.1`).
    pub fn new() -> RedisResult<Connection> {
        let uri = env::var("REDIS_SERVER").unwrap_or_else(|_| "redis://127.0.0.1".to_string());
        let client = redis::Client::open(uri.as_str())?;
        client.get_connection()
    }
}
