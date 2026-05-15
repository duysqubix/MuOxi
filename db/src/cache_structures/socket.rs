//! Per-client socket state stored in Redis.

use crate::cache::Cache;
use crate::cache_structures::Cachable;
use crate::utils::{UID, gen_uid};
use redis::{Commands, Connection, RedisResult, ToRedisArgs};
use std::net::SocketAddr;
use std::str::FromStr;
use std::string::ToString;

/// Per-connection socket state, persisted in Redis under
/// keys `Socket:UID:{ip,port,uid}`.
pub struct CacheSocket {
    conn: Connection,
    name: String,
    /// unique id of socket
    pub uid: UID,
    /// ip address of where socket is coming from
    pub ip: String,
    /// port of where socket is coming from
    pub port: u16,
}

impl CacheSocket {
    /// Construct with a known UID (existing client reconnecting).
    pub fn new_with_uid(uid: UID) -> Self {
        let conn = Cache::new().expect("Couldn't establish connection to caching server");
        Self {
            conn,
            name: String::from("Socket"),
            uid,
            ip: String::new(),
            port: 0,
        }
    }

    /// Construct with a freshly-generated UID.
    pub fn new() -> Self {
        Self::new_with_uid(gen_uid())
    }

    /// Set both ip and port from a `SocketAddr`.
    pub fn set_address(&mut self, addr: &SocketAddr) -> &mut Self {
        self.ip = addr.ip().to_string();
        self.port = addr.port();
        self
    }

    /// Set the ip field.
    pub fn set_ip(&mut self, ip: &str) -> &mut Self {
        self.ip = String::from(ip);
        self
    }

    /// Set the port field.
    pub fn set_port(&mut self, port: u16) -> &mut Self {
        self.port = port;
        self
    }

    /// Set a single field's value in Redis.
    pub fn set_value<F: ToRedisArgs + ToString, V: ToRedisArgs + ToString>(
        &mut self,
        field: F,
        value: V,
    ) -> RedisResult<()> {
        let tag = self.create_tag(field.to_string().as_str(), &value);
        let _: () = self.conn.set(tag.0, tag.1)?;
        Ok(())
    }

    /// Retrieve a single field's value, parsed as `T`.
    pub fn get_value<T: FromStr + std::fmt::Debug>(&mut self, field_name: &str) -> Option<T>
    where
        T::Err: std::fmt::Debug,
    {
        let key = self.make_key(field_name);
        let raw: Option<String> = self.conn.get(key).ok();
        raw.map(|s| {
            s.parse::<T>()
                .expect("Couldn't parse result form redis to appropriate type")
        })
    }
}

impl Default for CacheSocket {
    fn default() -> Self {
        Self::new()
    }
}

impl Cachable for CacheSocket {
    fn dump(&mut self) -> RedisResult<()> {
        let ip = self.create_tag("ip", &self.ip);
        let port = self.create_tag("port", &self.port);
        let uid = self.create_tag("uid", &self.uid);
        let _: () = self.conn.mset(&[ip, port, uid])?;
        Ok(())
    }

    fn load(mut self) -> RedisResult<Self> {
        let ip: String = self.conn.get(self.make_key("ip"))?;
        let port: String = self.conn.get(self.make_key("port"))?;
        self.ip = ip;
        self.port = port
            .parse::<u16>()
            .expect("Couldn't parse `port` to a u16 when deserializing from redis.");
        Ok(self)
    }

    fn destruct(mut self) -> RedisResult<()> {
        let _: () = self.conn.del(self.make_key("ip"))?;
        let _: () = self.conn.del(self.make_key("port"))?;
        let _: () = self.conn.del(self.make_key("uid"))?;
        Ok(())
    }

    fn uid(&self) -> UID {
        self.uid
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }
}
