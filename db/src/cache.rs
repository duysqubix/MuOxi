//!
//! Wrapper around redis server to easy manipulate cached data
//!
//! ### Example
//! ```ignore
//! use db::cache::Cache;
//! use redis::*;
//!
//! let mut cache = Cache::new().expect("Can't connect to redis server");
//!
//! // set key-value
//! let () = cache.conn.set("my_key", 42).unwrap();
//!
//! // get key-value, key exists.
//! let value: Option<usize> = cache.conn.get("my_key").unwrap(); // Some(42)
//!
//! // get key-value, key does not exist
//! let value: Option<usize> = cache.conn.get("other_key").unwrap(); // None
//! ```
//!

use redis::{Connection, RedisResult};

/// default address for redis server
pub static REDIS_SERVER: &'static str = "redis://127.0.0.1";

/// main wrapper around redis::Connection
pub struct Cache;

impl Cache {
    /// create new connection to cache server using custom uri
    pub fn new_with_uri<'a>(uri: &'a str) -> RedisResult<Connection> {
        let client = redis::Client::open(uri)?;
        let conn = client.get_connection()?;
        Ok(conn)
    }

    /// create new connection to cache server using default uri
    pub fn new() -> RedisResult<Connection> {
        let client = redis::Client::open(REDIS_SERVER)?;
        let conn = client.get_connection()?;
        Ok(conn)
    }
}
