#![deny(missing_docs)]

//! Helpers for storing rust structs in the Redis cache.
//!
//! Implementors of [`Cachable`] need at minimum a `redis::Connection` field
//! and a `UID` field. The serialization scheme uses the redis key format
//! `Type:UID:fieldName -> fieldValue` (one redis key per field) so individual
//! fields can carry per-key TTLs - something a single hash entry cannot do.

pub mod socket;

use crate::utils::UID;
use redis::{RedisResult, ToRedisArgs};
use std::string::ToString;

/// Trait implemented by struct types that round-trip through Redis.
pub trait Cachable {
    /// Persist the struct's fields to Redis.
    fn dump(&mut self) -> RedisResult<()>;

    /// Reload the struct's fields from Redis (UID must already be set).
    fn load(self) -> RedisResult<Self>
    where
        Self: Sized;

    /// Permanently remove all fields associated with this object from Redis.
    fn destruct(self) -> RedisResult<()>;

    /// Return the object's UID.
    fn uid(&self) -> UID;

    /// Return the object's type name (used as the redis key prefix).
    fn name(&self) -> &str;

    /// Build a redis key from the type name, UID, and field name.
    fn make_key(&self, field_name: &str) -> String {
        format!("{}:{}:{}", self.name(), self.uid(), field_name)
    }

    /// Build a `(key, value)` pair ready for `Connection::set` / `set_multiple`.
    fn create_tag<T: ToRedisArgs + ToString>(
        &self,
        field_name: &str,
        field_value: &T,
    ) -> (String, String) {
        (self.make_key(field_name), field_value.to_string())
    }
}
