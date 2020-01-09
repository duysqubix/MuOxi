//!
//! Crate to help with interaction with redis server
//!

use redis::{Connection, RedisResult};

pub static REDIS_SERVER: &'static str = "redis://127.0.0.1";

pub struct Cache {
    pub conn: Connection,
}

impl Cache {
    pub fn new_with_uri<'a>(uri: &'a str) -> RedisResult<Self> {
        let client = redis::Client::open(uri)?;
        let conn = client.get_connection()?;
        Ok(Self { conn: conn })
    }

    pub fn new() -> RedisResult<Self> {
        let client = redis::Client::open(REDIS_SERVER)?;
        let conn = client.get_connection()?;
        Ok(Self { conn: conn })
    }
}
