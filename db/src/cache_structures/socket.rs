//!
//! Holds structure for Socket information and manipulation
//! in caching server.
//!

use crate::cache::Cache;
use crate::cache_structures::Cachable;
use crate::utils::{gen_uid, UID};
use redis::{Commands, Connection, RedisResult};
use std::borrow::ToOwned;

static CACHE_SOCKET: &'static str = "CacheSocket";

/// Template structure to for raw socket
/// information
pub struct CacheSocket {
    /// raw connection to cache server
    conn: Connection,

    /// unique id of socket
    pub uid: UID,

    /// ip address of where socket is coming form
    pub ip: String,

    /// port of where socket is coming from
    pub port: u32,
}

impl<'a> CacheSocket {
    /// create blank instance of Socket
    pub fn new() -> Self {
        let conn = Cache::new().expect("Couldn't establish connection to caching server");
        Self {
            conn: conn,
            uid: gen_uid(),
            ip: String::new(),
            port: 0,
        }
    }

    /// set ip for this struct
    pub fn set_ip(&mut self, ip: &'a str) -> &mut Self {
        self.ip = String::from(ip);
        self
    }

    /// set port for this struct
    pub fn set_port(&mut self, port: u32) -> &mut Self {
        self.port = port;
        self
    }
}

impl Cachable for CacheSocket {
    fn dump(&mut self) -> RedisResult<()> {
        // convert current structure to to be fed into redis::hset_multiple
        let key = self.make_key(CACHE_SOCKET, self.uid);
        let ip = self.make_item("ip", &self.ip);
        let port = self.make_item("port", self.port);
        self.conn.hset(&key, ip.0, ip.1)?;
        self.conn.hset(&key, port.0, port.1)?;
        println!("{}", key);
        Ok(())
    }

    fn load(mut self) -> RedisResult<Self> {
        let key = self.make_key(CACHE_SOCKET, self.uid);
        let ip: String = self.conn.hget(&key, "ip")?;
        let port: u32 = self.conn.hget(&key, "port")?;

        self.ip = ip;
        self.port = port;
        Ok(self)
    }
}
