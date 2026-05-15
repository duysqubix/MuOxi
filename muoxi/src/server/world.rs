//! Thin facade over `db::DatabaseHandler` for command handlers.
//!
//! Commands receive a `&WorldApi`. Direct Diesel access is not exposed.

use db::DatabaseHandler;
use db::diesel::QueryResult;
use db::diesel::prelude::*;
use db::objects::Object;
use db::structures::account::Account;
use db::utils::{UID, gen_uid};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
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

    /// Create a new account. Returns the inserted row.
    /// Fails with "name probably taken" if the unique-name constraint fires.
    pub async fn create_account(
        &self,
        name: &str,
        password_hash: &str,
        email: &str,
    ) -> Result<Account, &'static str> {
        let uid = gen_uid();
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let mut db = self.db.lock().await;
        use db::schema::accounts;
        let row = Account {
            uid,
            name: name.to_string(),
            password_hash: password_hash.to_string(),
            email: email.to_string(),
            created_at,
        };
        db::diesel::insert_into(accounts::table)
            .values(&row)
            .execute(&mut db.handle)
            .map_err(|_| "insert failed (name probably taken)")?;
        Ok(row)
    }

    /// Find the seeded starting room (tagged `(starting-room, system)` by
    /// `crate::seed::seed_world`). `None` before the first boot's seed pass
    /// has run.
    pub async fn starting_room(&self) -> Option<UID> {
        self.objects_with_tag(
            crate::seed::STARTING_ROOM_TAG.0,
            crate::seed::STARTING_ROOM_TAG.1,
        )
        .await
        .ok()
        .and_then(|v| v.into_iter().next())
    }

    /// List the character objects belonging to `account_uid`, in ordinal order.
    pub async fn list_account_characters(&self, account_uid: UID) -> Vec<Object> {
        let mut db = self.db.lock().await;
        let DatabaseHandler {
            handle,
            character_accounts,
            objects,
            ..
        } = &mut *db;
        let links = character_accounts
            .list_for_account(handle, account_uid)
            .unwrap_or_default();
        let mut out = Vec::with_capacity(links.len());
        for link in links {
            if let Ok(Some(obj)) = objects.get(handle, link.object_uid) {
                out.push(obj);
            }
        }
        out
    }

    /// Create a character object owned by `account_uid`, named `name`, placed
    /// in `location_uid`. Applies the registered `character` TypeClass's
    /// default attributes and tags.
    pub async fn create_character(
        &self,
        registry: &crate::registry::Registry,
        account_uid: UID,
        name: &str,
        location_uid: Option<UID>,
    ) -> Result<Object, &'static str> {
        let mut db = self.db.lock().await;
        let DatabaseHandler {
            handle,
            objects,
            attributes,
            tags,
            character_accounts,
            ..
        } = &mut *db;

        let obj = objects
            .create(handle, "character", name, location_uid)
            .map_err(|_| "could not create character object")?;
        character_accounts
            .link(handle, obj.uid, account_uid)
            .map_err(|_| "could not link character to account")?;

        if let Some(tc) = registry.get_type("character") {
            for (k, v) in tc.default_attributes() {
                let _ = attributes.set(handle, obj.uid, &k, &v);
            }
            for (k, cat) in tc.default_tags() {
                let _ = tags.add(handle, obj.uid, &k, &cat);
            }
        }
        Ok(obj)
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
