//! `ObjectAttribute` model and `AttributeRepo`.
//!
//! Values are stored as JSON-encoded `TEXT` in the database. The Rust API
//! takes and returns `serde_json::Value` so callers get typed access without
//! locking the schema into one shape.

use crate::conn::Conn;
use crate::schema::object_attributes;
use crate::utils::UID;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single attribute row.
#[derive(Queryable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = object_attributes)]
pub struct ObjectAttribute {
    /// owning object's uid
    pub object_uid: UID,
    /// attribute key (per-object unique)
    pub key: String,
    /// JSON-encoded value
    pub value: String,
}

/// CRUD on the `object_attributes` table.
pub struct AttributeRepo;

impl AttributeRepo {
    /// Set or replace an attribute. Serializes `value` to JSON text.
    pub fn set(
        &self,
        conn: &mut Conn,
        object_uid: UID,
        key: &str,
        value: &serde_json::Value,
    ) -> QueryResult<usize> {
        let serialized = serde_json::to_string(value)
            .map_err(|e| diesel::result::Error::SerializationError(Box::new(e)))?;
        let row = ObjectAttribute {
            object_uid,
            key: key.to_string(),
            value: serialized,
        };
        diesel::insert_into(object_attributes::table)
            .values(&row)
            .on_conflict((object_attributes::object_uid, object_attributes::key))
            .do_update()
            .set(object_attributes::value.eq(&row.value))
            .execute(conn)
    }

    /// Get an attribute and parse its JSON. `None` if the key doesn't exist.
    pub fn get(
        &self,
        conn: &mut Conn,
        object_uid: UID,
        key: &str,
    ) -> QueryResult<Option<serde_json::Value>> {
        let row: Option<ObjectAttribute> = object_attributes::table
            .filter(object_attributes::object_uid.eq(object_uid))
            .filter(object_attributes::key.eq(key))
            .first::<ObjectAttribute>(conn)
            .optional()?;
        match row {
            Some(r) => {
                let v = serde_json::from_str(&r.value)
                    .map_err(|e| diesel::result::Error::DeserializationError(Box::new(e)))?;
                Ok(Some(v))
            }
            None => Ok(None),
        }
    }

    /// Delete an attribute by key. Returns rows affected (0 or 1).
    pub fn delete(&self, conn: &mut Conn, object_uid: UID, key: &str) -> QueryResult<usize> {
        diesel::delete(
            object_attributes::table
                .filter(object_attributes::object_uid.eq(object_uid))
                .filter(object_attributes::key.eq(key)),
        )
        .execute(conn)
    }

    /// Load all attributes of an object as a `HashMap<key, parsed JSON>`.
    pub fn all(
        &self,
        conn: &mut Conn,
        object_uid: UID,
    ) -> QueryResult<HashMap<String, serde_json::Value>> {
        let rows: Vec<ObjectAttribute> = object_attributes::table
            .filter(object_attributes::object_uid.eq(object_uid))
            .load::<ObjectAttribute>(conn)?;
        let mut out = HashMap::with_capacity(rows.len());
        for r in rows {
            let v = serde_json::from_str(&r.value)
                .map_err(|e| diesel::result::Error::DeserializationError(Box::new(e)))?;
            out.insert(r.key, v);
        }
        Ok(out)
    }
}
