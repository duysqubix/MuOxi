//! `ObjectTag` model and `TagRepo`.

use crate::conn::Conn;
use crate::schema::object_tags;
use crate::utils::UID;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A tag row.
#[derive(Queryable, Insertable, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[diesel(table_name = object_tags)]
pub struct ObjectTag {
    /// owning object
    pub object_uid: UID,
    /// tag label
    pub key: String,
    /// optional grouping category; empty string means "no category"
    pub category: String,
}

/// CRUD on the `object_tags` table.
pub struct TagRepo;

impl TagRepo {
    /// Add a tag. Idempotent: re-adding the same (key, category) pair does nothing.
    pub fn add(
        &self,
        conn: &mut Conn,
        object_uid: UID,
        key: &str,
        category: &str,
    ) -> QueryResult<usize> {
        let row = ObjectTag {
            object_uid,
            key: key.to_string(),
            category: category.to_string(),
        };
        diesel::insert_into(object_tags::table)
            .values(&row)
            .on_conflict((
                object_tags::object_uid,
                object_tags::key,
                object_tags::category,
            ))
            .do_nothing()
            .execute(conn)
    }

    /// Remove one tag. Returns rows affected.
    pub fn remove(
        &self,
        conn: &mut Conn,
        object_uid: UID,
        key: &str,
        category: &str,
    ) -> QueryResult<usize> {
        diesel::delete(
            object_tags::table
                .filter(object_tags::object_uid.eq(object_uid))
                .filter(object_tags::key.eq(key))
                .filter(object_tags::category.eq(category)),
        )
        .execute(conn)
    }

    /// True if the tag exists on the object.
    pub fn has(
        &self,
        conn: &mut Conn,
        object_uid: UID,
        key: &str,
        category: &str,
    ) -> QueryResult<bool> {
        let count: i64 = object_tags::table
            .filter(object_tags::object_uid.eq(object_uid))
            .filter(object_tags::key.eq(key))
            .filter(object_tags::category.eq(category))
            .count()
            .get_result(conn)?;
        Ok(count > 0)
    }

    /// Find all object UIDs carrying `(key, category)`.
    pub fn objects_with(
        &self,
        conn: &mut Conn,
        key: &str,
        category: &str,
    ) -> QueryResult<Vec<UID>> {
        object_tags::table
            .filter(object_tags::key.eq(key))
            .filter(object_tags::category.eq(category))
            .select(object_tags::object_uid)
            .load::<UID>(conn)
    }

    /// All tags on an object.
    pub fn all(&self, conn: &mut Conn, object_uid: UID) -> QueryResult<Vec<ObjectTag>> {
        object_tags::table
            .filter(object_tags::object_uid.eq(object_uid))
            .load::<ObjectTag>(conn)
    }
}
