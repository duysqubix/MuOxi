#![deny(missing_docs)]
//!
//! Collection of utilies that define structures to store in caching server
//! as well as parsing those objects for server consumption.cache_structures
//!
//! For every wrapper around a rust object that you would like to serialize
//! into the caching server needs to have at least two fields to make it possible
//!
//! ### Example
//! ```ignore
//! use crate::utils::UID;
//!
//! // each struct will begin with *Cache[Name]*
//! pub struct CacheMyStruct{
//!     conn: Cache // <- wrapper of Connection to cache server
//!     uid: UID // <- must include a unique id number
//! }
//! ```
//! All other fields within the struct must be wrapped with `Option<MyField>`

pub mod socket;

use crate::cache::Cache;
use crate::utils::UID;
use redis::{RedisResult, ToRedisArgs};

/// trait to all structures that are cachable to cache server
pub trait Cachable {
    /// construct hmap of current object that inherits this trait
    fn dump(&mut self) -> RedisResult<()>;

    /// decontructs data from caching server and updates internal fields
    /// must not lose track of UID
    fn load(self) -> RedisResult<Self>
    where
        Self: std::marker::Sized;

    /// return hmap key with correct naming convention
    fn make_key<'a>(&self, name: &'a str, uid: UID) -> String {
        format!("{}:{}", name, uid)
    }

    /// make a redis compatible tuple for field:value
    fn make_item<F: ToRedisArgs, V: ToRedisArgs>(&self, field: F, value: V) -> (F, V) {
        (field, value)
    }
}
