#![deny(missing_docs)]
//!
//! Collection of utilies that define structures to store in caching server
//! as well as parsing those objects for server consumption
//!
//! For every wrapper around a rust object that you would like to serialize
//! into the caching server needs to have at least two fields to make it possible
//! 1) redis::Connection
//! 2) UID
//!
//! Defining this trait on a struct will allow methods to become available where
//! you can serialize/deserialize your rust object into redis server.Connection
//!
//! The naming scheme is pretty simple, as it uses the following format.
//!
//! `MyStruct:UID:fieldName fieldValue`
//!
//! this simple use of redis `get/set` allows individual fields to contain meta data
//! whereby `hmap` will not allow, such as expirary times for fields etc..
//!

pub mod socket;

use crate::utils::UID;
use redis::{RedisResult, ToRedisArgs};
use std::string::ToString;

/// trait to all structures that are cachable to cache server
pub trait Cachable {
    /// construct hmap of current object that inherits this trait
    fn dump(&mut self) -> RedisResult<()>;

    /// decontructs data from caching server and updates internal fields
    /// must not lose track of UID
    fn load(self) -> RedisResult<Self>
    where
        Self: std::marker::Sized;

    /// Permentatly removes all data associated to this object from redis server
    fn destruct(self) -> RedisResult<()>;

    /// retrieve UID for object
    fn uid(&self) -> UID;

    /// retrieve name of object
    fn name(&self) -> &str;

    /// create key from struct name, uid, and custom identifier
    fn make_key<'a>(&self, field_name: &'a str) -> String {
        format!("{}:{}:{}", self.name(), self.uid(), field_name)
    }

    /// Create tag for redis consumption in the format of
    /// `(Name:uid:field, value)`
    fn create_tag<'a, T: ToRedisArgs + ToString>(
        &self,
        field_name: &'a str,
        field_value: &'a T,
    ) -> (String, String) {
        (self.make_key(field_name), field_value.to_string())
    }
}
