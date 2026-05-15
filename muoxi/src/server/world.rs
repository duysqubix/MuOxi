//! Thin facade over `db::DatabaseHandler` for command handlers.
//!
//! Commands receive a `&WorldApi`. Direct Diesel access is not exposed.

use db::DatabaseHandler;
use db::diesel::QueryResult;
use db::diesel::prelude::*;
use db::objects::Object;
use db::structures::account::Account;
use db::utils::UID;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Database facade for the engine. Wraps the connection in a Tokio mutex so
/// async command handlers can serialize access without blocking the runtime.
pub struct WorldApi {
    db: Arc<Mutex<DatabaseHandler>>,
}

impl WorldApi {
    /// Construct from an owned `DatabaseHandler`.
    pub fn new(db: DatabaseHandler) -> Self {
        Self {
            db: Arc::new(Mutex::new(db)),
        }
    }

    /// Create an object. Hooks (`at_object_created`) are fired by the caller
    /// (the engine), not by this method, to keep the locked region small.
    pub async fn create_object(
        &self,
        type_key: &str,
        name: &str,
        location: Option<UID>,
    ) -> QueryResult<Object> {
        let mut db = self.db.lock().await;
        let DatabaseHandler { handle, objects, .. } = &mut *db;
        objects.create(handle, type_key, name, location)
    }

    /// Get an object by uid.
    pub async fn get_object(&self, uid: UID) -> QueryResult<Option<Object>> {
        let mut db = self.db.lock().await;
        let DatabaseHandler { handle, objects, .. } = &mut *db;
        objects.get(handle, uid)
    }

    /// Move an object.
    pub async fn move_object(&self, uid: UID, dest: Option<UID>) -> QueryResult<usize> {
        let mut db = self.db.lock().await;
        let DatabaseHandler { handle, objects, .. } = &mut *db;
        objects.move_to(handle, uid, dest)
    }

    /// Set an attribute (JSON-serialized).
    pub async fn set_attribute(
        &self,
        uid: UID,
        key: &str,
        value: serde_json::Value,
    ) -> QueryResult<usize> {
        let mut db = self.db.lock().await;
        let DatabaseHandler { handle, attributes, .. } = &mut *db;
        attributes.set(handle, uid, key, &value)
    }

    /// Get an attribute.
    pub async fn get_attribute(
        &self,
        uid: UID,
        key: &str,
    ) -> QueryResult<Option<serde_json::Value>> {
        let mut db = self.db.lock().await;
        let DatabaseHandler { handle, attributes, .. } = &mut *db;
        attributes.get(handle, uid, key)
    }

    /// True if `target` has tag `(key, category)`.
    pub async fn has_tag(&self, target: UID, key: &str, category: &str) -> QueryResult<bool> {
        let mut db = self.db.lock().await;
        let DatabaseHandler { handle, tags, .. } = &mut *db;
        tags.has(handle, target, key, category)
    }

    /// Add a tag.
    pub async fn add_tag(&self, target: UID, key: &str, category: &str) -> QueryResult<usize> {
        let mut db = self.db.lock().await;
        let DatabaseHandler { handle, tags, .. } = &mut *db;
        tags.add(handle, target, key, category)
    }

    /// Find all object UIDs carrying a (key, category) tag.
    pub async fn objects_with_tag(&self, key: &str, category: &str) -> QueryResult<Vec<UID>> {
        let mut db = self.db.lock().await;
        let DatabaseHandler { handle, tags, .. } = &mut *db;
        tags.objects_with(handle, key, category)
    }

    /// List the contents of a container/room.
    pub async fn contents_of(&self, location: UID) -> QueryResult<Vec<Object>> {
        let mut db = self.db.lock().await;
        let DatabaseHandler { handle, objects, .. } = &mut *db;
        objects.contents_of(handle, location)
    }

    /// Look up an account by name (case-sensitive). `None` if absent.
    pub async fn find_account_by_name(&self, name: &str) -> Option<Account> {
        let mut db = self.db.lock().await;
        use db::schema::accounts::dsl;
        dsl::accounts
            .filter(dsl::name.eq(name))
            .first::<Account>(&mut db.handle)
            .ok()
    }

    /// Get the stored password hash for an account UID. `None` if the account
    /// no longer exists.
    pub async fn account_password_hash(&self, account_uid: UID) -> Option<String> {
        let mut db = self.db.lock().await;
        use db::schema::accounts::dsl;
        dsl::accounts
            .filter(dsl::uid.eq(account_uid))
            .select(dsl::password_hash)
            .first::<String>(&mut db.handle)
            .ok()
    }

    /// Inner DB access for advanced callers (still locks).
    pub async fn with_db<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut DatabaseHandler) -> T,
    {
        let mut db = self.db.lock().await;
        f(&mut db)
    }
}
