#![deny(missing_docs)]

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

/// main wrapper around redis::Connection
pub struct Cache;

use std::env;

impl Cache {
    /// create new connection to cache server using custom uri
    pub fn new_with_uri<'a>(uri: &'a str) -> RedisResult<Connection> {
        let client = redis::Client::open(uri)?;
        let conn = client.get_connection()?;
        Ok(conn)
    }

    /// create new connection to cache server using default uri
    pub fn new() -> RedisResult<Connection> {
        let redis_uri = env::var("REDIS_SERVER").unwrap_or("redis://127.0.0.1".to_string());
        let client = redis::Client::open(redis_uri.as_str())?;
        let conn = client.get_connection()?;
        Ok(conn)
    }
}
