//! `Object` model and `ObjectRepo`.

use crate::conn::Conn;
use crate::schema::objects;
use crate::utils::{UID, gen_uid};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// A generic in-world entity row.
#[derive(Queryable, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Object {
    /// unique id
    pub uid: UID,
    /// type discriminator: "character", "room", "item", "exit", or downstream-defined
    pub type_key: String,
    /// human-display name
    pub name: String,
    /// containing object's uid (rooms have None; items have a room or an inventory)
    pub location_uid: Option<UID>,
    /// unix epoch seconds at creation
    pub created_at: i64,
}

/// Insert payload for creating an object.
#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = objects)]
pub struct NewObject<'a> {
    /// pre-allocated UID (use `gen_uid()`)
    pub uid: UID,
    /// see `Object::type_key`
    pub type_key: &'a str,
    /// see `Object::name`
    pub name: &'a str,
    /// see `Object::location_uid`
    pub location_uid: Option<UID>,
    /// unix epoch seconds
    pub created_at: i64,
}

/// AsChangeset payload for updating an object's mutable fields.
#[derive(AsChangeset, Debug, Clone)]
#[diesel(table_name = objects)]
pub struct ObjectUpdate<'a> {
    /// new name (None = unchanged)
    pub name: Option<&'a str>,
    /// new location (Some(None) clears, Some(Some(_)) sets, None leaves unchanged)
    pub location_uid: Option<Option<UID>>,
}

/// CRUD on the `objects` table.
pub struct ObjectRepo;

impl ObjectRepo {
    /// Create a new object with a fresh UID. Returns the persisted row.
    pub fn create(
        &self,
        conn: &mut Conn,
        type_key: &str,
        name: &str,
        location_uid: Option<UID>,
    ) -> QueryResult<Object> {
        let uid = gen_uid();
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let row = NewObject {
            uid,
            type_key,
            name,
            location_uid,
            created_at,
        };
        diesel::insert_into(objects::table)
            .values(&row)
            .execute(conn)?;
        self.get(conn, uid)?.ok_or(diesel::result::Error::NotFound)
    }

    /// Get a single object by UID.
    pub fn get(&self, conn: &mut Conn, uid: UID) -> QueryResult<Option<Object>> {
        objects::table
            .filter(objects::uid.eq(uid))
            .first::<Object>(conn)
            .optional()
    }

    /// Delete an object (cascades to attributes, tags, character_accounts via FK).
    pub fn delete(&self, conn: &mut Conn, uid: UID) -> QueryResult<usize> {
        diesel::delete(objects::table.filter(objects::uid.eq(uid))).execute(conn)
    }

    /// Move `uid` to be located inside `new_location` (or no location if None).
    pub fn move_to(
        &self,
        conn: &mut Conn,
        uid: UID,
        new_location: Option<UID>,
    ) -> QueryResult<usize> {
        diesel::update(objects::table.filter(objects::uid.eq(uid)))
            .set(objects::location_uid.eq(new_location))
            .execute(conn)
    }

    /// Rename an object.
    pub fn rename(&self, conn: &mut Conn, uid: UID, new_name: &str) -> QueryResult<usize> {
        diesel::update(objects::table.filter(objects::uid.eq(uid)))
            .set(objects::name.eq(new_name))
            .execute(conn)
    }

    /// All objects of a given type.
    pub fn list_by_type(&self, conn: &mut Conn, type_key: &str) -> QueryResult<Vec<Object>> {
        objects::table
            .filter(objects::type_key.eq(type_key))
            .load::<Object>(conn)
    }

    /// All objects whose `location_uid` equals `location` (the contents of a container/room).
    pub fn contents_of(&self, conn: &mut Conn, location: UID) -> QueryResult<Vec<Object>> {
        objects::table
            .filter(objects::location_uid.eq(Some(location)))
            .load::<Object>(conn)
    }
}
