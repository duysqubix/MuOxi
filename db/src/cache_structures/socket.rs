//!
//! Holds structure for Socket information and manipulation
//! in caching server.
//!

use crate::cache::Cache;
use crate::cache_structures::Cachable;
use crate::utils::{gen_uid, UID};
use redis::{Commands, Connection, RedisResult, ToRedisArgs};
use std::net::SocketAddr;
use std::str::FromStr;
use std::string::ToString;

/// Template structure to for raw socket
/// information
pub struct CacheSocket {
    /// raw connection to cache server
    conn: Connection,

    //name of struct, to be used as first parameter in redis storage
    name: String,

    /// unique id of socket
    pub uid: UID,

    /// ip address of where socket is coming form
    pub ip: String,

    /// port of where socket is coming from
    pub port: u16,
}

impl<'a> CacheSocket {
    /// create instance of socket with supplied UID
    pub fn new_with_uid(uid: UID) -> Self {
        let conn = Cache::new().expect("Couldn't establish connection to caching server");
        Self {
            conn: conn,
            name: String::from("Socket"),
            uid: uid,
            ip: String::new(),
            port: 0,
        }
    }

    /// create blank instance of Socket
    pub fn new() -> Self {
        let conn = Cache::new().expect("Couldn't establish connection to caching server");
        Self {
            conn: conn,
            name: String::from("Socket"),
            uid: gen_uid(),
            ip: String::new(),
            port: 0,
        }
    }
    /// set ip and port for this struct with a supplied `std::net::SocketAddr`
    pub fn set_address(&mut self, addr: &SocketAddr) -> &mut Self {
        let ip = format!("{}", addr.ip());
        self.ip = String::from(ip);
        self.port = addr.port();
        self
    }

    /// set ip for this struct
    pub fn set_ip(&mut self, ip: &'a str) -> &mut Self {
        self.ip = String::from(ip);
        self
    }

    /// set port for this struct
    pub fn set_port(&mut self, port: u16) -> &mut Self {
        self.port = port;
        self
    }

    /// set key and value
    pub fn set_value<F: ToRedisArgs + ToString, V: ToRedisArgs + ToString>(
        &mut self,
        field: F,
        value: V,
    ) -> RedisResult<()> {
        let tag = self.create_tag(field.to_string().as_str(), &value);
        self.conn.set(tag.0, tag.1)?;
        Ok(())
    }

    /// retrieve a value from field of struct, if it does not exist, will return None
    pub fn get_value<T: FromStr + std::fmt::Debug>(&mut self, field_name: &'a str) -> Option<T>
    where
        T::Err: std::fmt::Debug,
    {
        let key = self.make_key(field_name);

        let result: Option<String> = match self.conn.get(key) {
            Ok(result) => Some(result),
            Err(_) => None,
        };

        if result.is_some() {
            return Some(
                result
                    .unwrap()
                    .parse::<T>()
                    .expect("Couldn't parse result form redis to appropriate type"),
            );
        } else {
            return None;
        }
    }
}

impl Cachable for CacheSocket {
    fn dump(&mut self) -> RedisResult<()> {
        // convert current structure to to be fed into redis::hset_multiple
        // let key = self.make_key(CACHE_SOCKET, self.);
        let ip = self.create_tag("ip", &self.ip);
        let port = self.create_tag("port", &self.port);
        let uid = self.create_tag("uid", &self.uid);
        self.conn.set_multiple(&vec![ip, port, uid])?;

        Ok(())
    }

    fn load(mut self) -> RedisResult<Self> {
        let ip: String = self.conn.get(self.make_key("ip"))?;
        let port: String = self.conn.get(self.make_key("port"))?;
        self.ip = ip;
        self.port = port
            .parse::<u16>()
            .expect("Couldn't not parse `port` to a number when deserializing from redis.");
        Ok(self)
    }

    fn destruct(mut self) -> RedisResult<()> {
        let ip = self.make_key("ip");
        let port = self.make_key("port");
        let uid = self.make_key("uid");

        self.conn.del(ip)?;
        self.conn.del(port)?;
        self.conn.del(uid)?;

        Ok(())
    }
    fn uid(&self) -> UID {
        self.uid
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }
}
