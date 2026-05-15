# Generic Object / Attribute / Tag Model Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add the Evennia-style generic in-world object model so downstream framework users can build rooms / items / mobs / exits / characters as `Object` rows with freeform `Attribute` and `Tag` data, without schema migrations.

**Architecture:** Three new tables — `objects` (typed in-world entities), `object_attributes` (per-object key→JSON-text), `object_tags` (per-object key+category labels). The standalone `characters` table is replaced by objects with `type_key = "character"`, linked to accounts via a new `character_accounts` join table. Accounts remain as a separate typed table because login identity is fundamentally different from in-world entities. Repository structs (`ObjectRepo`, `AttributeRepo`, `TagRepo`, `CharacterAccountRepo`) wrap Diesel queries so engine code never imports `diesel::*` directly.

**Tech Stack:** Diesel 2.2 (sqlite default + postgres feature), `serde_json` for attribute values, no new external deps.

---

## File Structure

**Create:**
- `migrations/2026-05-07-000100_objects/up.sql`
- `migrations/2026-05-07-000100_objects/down.sql`
- `db/src/objects/mod.rs` — module root, re-exports
- `db/src/objects/object.rs` — `Object` model + `ObjectRepo`
- `db/src/objects/attribute.rs` — `ObjectAttribute` model + `AttributeRepo`
- `db/src/objects/tag.rs` — `ObjectTag` model + `TagRepo`
- `db/src/objects/character_account.rs` — `CharacterAccount` model + `CharacterAccountRepo`
- `db/tests/integration_objects.rs` — round-trip tests for all four repos

**Modify:**
- `db/src/schema.rs` — add `objects`, `object_attributes`, `object_tags`, `character_accounts`; remove `characters`, `account_characters`
- `db/src/structures.rs` — drop `character` module, drop `account_character_link` module; keep `account` module
- `db/src/lib.rs` — expose new `objects` module; add `objects: ObjectRepo` etc. fields to `DatabaseHandler`
- root `AGENTS.md` and `db/AGENTS.md` — document new schema

**Delete:** none in this plan (only schema-level deletes via migration).

---

## Task 1: Write the migration

**Files:**
- Create: `migrations/2026-05-07-000100_objects/up.sql`
- Create: `migrations/2026-05-07-000100_objects/down.sql`

- [ ] **Step 1: Write `up.sql`**

```sql
-- objects: a generic in-world entity. type_key discriminates rooms, items,
-- characters, mobs, exits, etc. Downstream framework users register their
-- own type_keys; the framework treats them all uniformly.
CREATE TABLE objects (
    uid          BIGINT  NOT NULL CHECK (uid > 0),
    type_key     TEXT    NOT NULL,
    name         TEXT    NOT NULL,
    location_uid BIGINT  NULL,
    created_at   BIGINT  NOT NULL,
    PRIMARY KEY (uid),
    FOREIGN KEY (location_uid) REFERENCES objects(uid) ON DELETE SET NULL
);

CREATE INDEX idx_objects_type_key ON objects(type_key);
CREATE INDEX idx_objects_location ON objects(location_uid);

-- object_attributes: per-object freeform key/value bag. value is a JSON-encoded
-- string (use serde_json::Value at the Rust layer). Avoids schema migration
-- for downstream gameplay state.
CREATE TABLE object_attributes (
    object_uid BIGINT NOT NULL,
    key        TEXT   NOT NULL,
    value      TEXT   NOT NULL,
    PRIMARY KEY (object_uid, key),
    FOREIGN KEY (object_uid) REFERENCES objects(uid) ON DELETE CASCADE
);

-- object_tags: labels with optional category. Used for grouping and lookups
-- ("all rooms tagged 'safe-zone'", "all objects with the 'pvp' permission").
CREATE TABLE object_tags (
    object_uid BIGINT NOT NULL,
    key        TEXT   NOT NULL,
    category   TEXT   NOT NULL DEFAULT '',
    PRIMARY KEY (object_uid, key, category),
    FOREIGN KEY (object_uid) REFERENCES objects(uid) ON DELETE CASCADE
);

CREATE INDEX idx_object_tags_lookup ON object_tags(category, key);

-- character_accounts: link table between objects (where type_key='character')
-- and the owning account. Replaces the old account_characters/characters
-- pairing.
CREATE TABLE character_accounts (
    object_uid  BIGINT  NOT NULL,
    account_uid BIGINT  NOT NULL,
    ordinal     INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (object_uid),
    FOREIGN KEY (object_uid) REFERENCES objects(uid) ON DELETE CASCADE,
    FOREIGN KEY (account_uid) REFERENCES accounts(uid) ON DELETE CASCADE
);

CREATE INDEX idx_character_accounts_account ON character_accounts(account_uid, ordinal);

-- Drop the now-redundant standalone tables. character data has migrated to
-- objects + character_accounts.
DROP TABLE IF EXISTS account_characters;
DROP TABLE IF EXISTS characters;
```

- [ ] **Step 2: Write `down.sql`**

```sql
-- Restore the previous schema. Loses object/attribute/tag data — that's the
-- nature of an undo for additive schemas.
CREATE TABLE characters (
    uid         BIGINT       NOT NULL CHECK (uid > 0),
    account_uid BIGINT       NOT NULL CHECK (account_uid > 0),
    name        VARCHAR(64)  NOT NULL UNIQUE,
    created_at  BIGINT       NOT NULL,
    PRIMARY KEY (uid),
    FOREIGN KEY (account_uid) REFERENCES accounts(uid) ON DELETE CASCADE
);

CREATE INDEX idx_characters_account ON characters(account_uid);

CREATE TABLE account_characters (
    account_uid   BIGINT  NOT NULL,
    character_uid BIGINT  NOT NULL,
    ordinal       INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_uid, character_uid),
    FOREIGN KEY (account_uid) REFERENCES accounts(uid) ON DELETE CASCADE,
    FOREIGN KEY (character_uid) REFERENCES characters(uid) ON DELETE CASCADE
);

DROP TABLE IF EXISTS character_accounts;
DROP TABLE IF EXISTS object_tags;
DROP TABLE IF EXISTS object_attributes;
DROP TABLE IF EXISTS objects;
```

- [ ] **Step 3: Verify the SQL parses on SQLite**

```bash
cd /home/duys/.repos/MuOxi
sqlite3 :memory: <<'EOF'
.read migrations/2026-05-07-000000_initial/up.sql
.read migrations/2026-05-07-000100_objects/up.sql
.tables
EOF
```

Expected output includes: `accounts  character_accounts  object_attributes  object_tags  objects`. (Skip if `sqlite3` is unavailable; Task 11 covers this via integration test.)

- [ ] **Step 4: Commit**

```bash
git add migrations/2026-05-07-000100_objects/
git commit -m "feat(db): migration adds objects/attributes/tags + character_accounts"
```

---

## Task 2: Update `db/src/schema.rs`

**Files:**
- Modify: `db/src/schema.rs`

- [ ] **Step 1: Replace the file with the new table macros**

```rust
diesel::table! {
    accounts (uid) {
        uid -> BigInt,
        name -> Text,
        password_hash -> Text,
        email -> Text,
        created_at -> BigInt,
    }
}

diesel::table! {
    objects (uid) {
        uid -> BigInt,
        type_key -> Text,
        name -> Text,
        location_uid -> Nullable<BigInt>,
        created_at -> BigInt,
    }
}

diesel::table! {
    object_attributes (object_uid, key) {
        object_uid -> BigInt,
        key -> Text,
        value -> Text,
    }
}

diesel::table! {
    object_tags (object_uid, key, category) {
        object_uid -> BigInt,
        key -> Text,
        category -> Text,
    }
}

diesel::table! {
    character_accounts (object_uid) {
        object_uid -> BigInt,
        account_uid -> BigInt,
        ordinal -> Integer,
    }
}

diesel::joinable!(object_attributes -> objects (object_uid));
diesel::joinable!(object_tags -> objects (object_uid));
diesel::joinable!(character_accounts -> objects (object_uid));
diesel::joinable!(character_accounts -> accounts (account_uid));

diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    objects,
    object_attributes,
    object_tags,
    character_accounts,
);
```

Note: the self-referential FK `objects.location_uid -> objects.uid` cannot use Diesel's `joinable!` macro (no self-joins via that macro). Self-joins use explicit `inner_join`/`left_join` calls or the `alias!` macro at query time.

- [ ] **Step 2: Verify**

Run: `cd /home/duys/.repos/MuOxi && cargo check -p db`
Expected: errors only from `structures.rs` referencing the removed `characters` and `account_characters` tables. Schema itself compiles.

- [ ] **Step 3: Commit**

```bash
git add db/src/schema.rs
git commit -m "feat(db): schema gains objects/attributes/tags; drops characters/account_characters"
```

---

## Task 3: Create `db/src/objects/mod.rs`

**Files:**
- Create: `db/src/objects/mod.rs`

- [ ] **Step 1: Write the module root**

```rust
//! Generic in-world object model.
//!
//! `Object` is the universal entity row (rooms, items, mobs, characters,
//! exits, downstream-defined types). `ObjectAttribute` is a freeform per-object
//! key/value JSON bag. `ObjectTag` is a per-object label with optional category.
//! `CharacterAccount` links character objects to login accounts.
//!
//! Engine code should go through the repository structs (`ObjectRepo`,
//! `AttributeRepo`, `TagRepo`, `CharacterAccountRepo`) and not import
//! `diesel::*` directly.

pub mod attribute;
pub mod character_account;
pub mod object;
pub mod tag;

pub use attribute::{AttributeRepo, ObjectAttribute};
pub use character_account::{CharacterAccount, CharacterAccountRepo};
pub use object::{NewObject, Object, ObjectRepo};
pub use tag::{ObjectTag, TagRepo};
```

- [ ] **Step 2: Commit**

```bash
git add db/src/objects/mod.rs
git commit -m "feat(db): add objects module root"
```

---

## Task 4: Create `Object` + `ObjectRepo`

**Files:**
- Create: `db/src/objects/object.rs`

- [ ] **Step 1: Write the module**

```rust
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
```

- [ ] **Step 2: Verify**

Run: `cd /home/duys/.repos/MuOxi && cargo check -p db`
Expected: still has errors from `structures.rs` (Task 8 fixes those), but `objects/object.rs` itself compiles.

- [ ] **Step 3: Commit**

```bash
git add db/src/objects/object.rs
git commit -m "feat(db): Object model + ObjectRepo with create/get/delete/move/list"
```

---

## Task 5: Create `ObjectAttribute` + `AttributeRepo`

**Files:**
- Create: `db/src/objects/attribute.rs`

- [ ] **Step 1: Write the module**

```rust
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
```

- [ ] **Step 2: Verify**

Run: `cd /home/duys/.repos/MuOxi && cargo check -p db`
Expected: same as Task 4.

- [ ] **Step 3: Commit**

```bash
git add db/src/objects/attribute.rs
git commit -m "feat(db): ObjectAttribute + AttributeRepo (JSON-text values)"
```

---

## Task 6: Create `ObjectTag` + `TagRepo`

**Files:**
- Create: `db/src/objects/tag.rs`

- [ ] **Step 1: Write the module**

```rust
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
```

- [ ] **Step 2: Verify**

Run: `cd /home/duys/.repos/MuOxi && cargo check -p db`
Expected: errors still from structures.rs only.

- [ ] **Step 3: Commit**

```bash
git add db/src/objects/tag.rs
git commit -m "feat(db): ObjectTag + TagRepo (idempotent add, lookup by category+key)"
```

---

## Task 7: Create `CharacterAccount` link + repo

**Files:**
- Create: `db/src/objects/character_account.rs`

- [ ] **Step 1: Write the module**

```rust
//! Link table between character objects (`objects.type_key = 'character'`)
//! and login accounts. Replaces the old `characters` + `account_characters` pair.

use crate::conn::Conn;
use crate::schema::character_accounts;
use crate::utils::UID;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A link row.
#[derive(Queryable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = character_accounts)]
pub struct CharacterAccount {
    /// uid of the character object (must reference `objects.uid` where `type_key = 'character'`)
    pub object_uid: UID,
    /// owning account uid
    pub account_uid: UID,
    /// 0-indexed position in this account's character list
    pub ordinal: i32,
}

/// CRUD on the `character_accounts` table.
pub struct CharacterAccountRepo;

impl CharacterAccountRepo {
    /// Link a character object to an account at the next available ordinal.
    pub fn link(
        &self,
        conn: &mut Conn,
        object_uid: UID,
        account_uid: UID,
    ) -> QueryResult<CharacterAccount> {
        let next_ordinal: i32 = character_accounts::table
            .filter(character_accounts::account_uid.eq(account_uid))
            .select(diesel::dsl::max(character_accounts::ordinal))
            .first::<Option<i32>>(conn)?
            .unwrap_or(-1)
            + 1;

        let row = CharacterAccount {
            object_uid,
            account_uid,
            ordinal: next_ordinal,
        };
        diesel::insert_into(character_accounts::table)
            .values(&row)
            .execute(conn)?;
        Ok(row)
    }

    /// Unlink a character (does NOT delete the underlying object — caller decides).
    pub fn unlink(&self, conn: &mut Conn, object_uid: UID) -> QueryResult<usize> {
        diesel::delete(
            character_accounts::table.filter(character_accounts::object_uid.eq(object_uid)),
        )
        .execute(conn)
    }

    /// All characters owned by `account_uid`, ordered by `ordinal`.
    pub fn list_for_account(
        &self,
        conn: &mut Conn,
        account_uid: UID,
    ) -> QueryResult<Vec<CharacterAccount>> {
        character_accounts::table
            .filter(character_accounts::account_uid.eq(account_uid))
            .order(character_accounts::ordinal.asc())
            .load::<CharacterAccount>(conn)
    }

    /// The account that owns a given character object, if any.
    pub fn owner_of(
        &self,
        conn: &mut Conn,
        object_uid: UID,
    ) -> QueryResult<Option<CharacterAccount>> {
        character_accounts::table
            .filter(character_accounts::object_uid.eq(object_uid))
            .first::<CharacterAccount>(conn)
            .optional()
    }
}
```

- [ ] **Step 2: Verify**

Run: `cd /home/duys/.repos/MuOxi && cargo check -p db`
Expected: errors only from structures.rs.

- [ ] **Step 3: Commit**

```bash
git add db/src/objects/character_account.rs
git commit -m "feat(db): CharacterAccount link + repo (replaces account_characters)"
```

---

## Task 8: Drop the `character` and `account_character_link` modules from `structures.rs`

**Files:**
- Modify: `db/src/structures.rs`

- [ ] **Step 1: Open `db/src/structures.rs` and delete the `character` module entirely**

The existing file has `pub mod character { ... }` (defines `Character`, `CharacterHandler`, `DatabaseHandlerExt<Character>`). Delete the whole module — character data lives in `Object` rows now.

- [ ] **Step 2: Delete the `account_character_link` module**

Same file, delete `pub mod account_character_link { ... }`. Replaced by `db::objects::CharacterAccountRepo`.

- [ ] **Step 3: Update the `Account` struct fields if needed**

The `Account` struct from Plan 1 already drops the `characters: Option<Vec<i64>>` field. Verify it has the four fields `uid, name, password_hash, email, created_at` and no others. If a stray characters field is still there, remove it.

- [ ] **Step 4: Verify**

Run: `cd /home/duys/.repos/MuOxi && cargo check -p db`
Expected: errors now only from `lib.rs` (it still references `characters: structures::character::CharacterHandler` field — Task 9 fixes that).

- [ ] **Step 5: Commit**

```bash
git add db/src/structures.rs
git commit -m "refactor(db): remove character + account_character_link modules"
```

---

## Task 9: Wire repos into `DatabaseHandler`

**Files:**
- Modify: `db/src/lib.rs`

- [ ] **Step 1: Replace the `DatabaseHandler` struct + impl**

```rust
//! Diesel-powered ORM management library for MuOxi.

pub mod cache;
pub mod cache_structures;
pub mod conn;
pub mod objects;
pub mod schema;
pub mod structures;
pub mod utils;

pub use conn::{Conn, configure, default_url, establish};

use objects::{AttributeRepo, CharacterAccountRepo, ObjectRepo, TagRepo};
use structures::account::AccountHandler;

/// Top-level database facade. Holds an open connection plus all repository
/// helpers. Construct with [`DatabaseHandler::connect`].
pub struct DatabaseHandler {
    /// active database connection
    pub handle: Conn,
    /// account-table CRUD
    pub accounts: AccountHandler,
    /// generic object CRUD
    pub objects: ObjectRepo,
    /// per-object attribute CRUD (JSON-text values)
    pub attributes: AttributeRepo,
    /// per-object tag CRUD
    pub tags: TagRepo,
    /// character⇄account link CRUD
    pub character_accounts: CharacterAccountRepo,
}

impl DatabaseHandler {
    /// Connect to the configured database and apply runtime pragmas.
    pub fn connect() -> Self {
        let mut handle = establish();
        configure(&mut handle).expect("configure() pragmas failed");
        Self {
            handle,
            accounts: AccountHandler,
            objects: ObjectRepo,
            attributes: AttributeRepo,
            tags: TagRepo,
            character_accounts: CharacterAccountRepo,
        }
    }
}
```

- [ ] **Step 2: Verify**

Run: `cd /home/duys/.repos/MuOxi && cargo check -p db`
Expected: `Finished` with no errors.

- [ ] **Step 3: Commit**

```bash
git add db/src/lib.rs
git commit -m "feat(db): DatabaseHandler exposes objects/attributes/tags/character_accounts"
```

---

## Task 10: Integration tests for the four repos

**Files:**
- Create: `db/tests/integration_objects.rs`

- [ ] **Step 1: Write the test file**

```rust
//! Round-trip tests for the object/attribute/tag/character_account repos.
//! Runs in-memory SQLite under the default `db-sqlite` feature.

#![cfg(feature = "db-sqlite")]

use db::objects::{AttributeRepo, CharacterAccountRepo, ObjectRepo, TagRepo};
use db::structures::account::{Account, AccountHandler};
use db::structures::DatabaseHandlerExt;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

const SCHEMA_INITIAL: &str = include_str!("../../migrations/2026-05-07-000000_initial/up.sql");
const SCHEMA_OBJECTS: &str = include_str!("../../migrations/2026-05-07-000100_objects/up.sql");

fn fresh_conn() -> SqliteConnection {
    let mut conn = SqliteConnection::establish(":memory:").expect("open memory sqlite");
    diesel::sql_query("PRAGMA foreign_keys = ON")
        .execute(&mut conn)
        .unwrap();
    for stmt in SCHEMA_INITIAL.split(';').chain(SCHEMA_OBJECTS.split(';')) {
        let trimmed = stmt.trim();
        if trimmed.is_empty() {
            continue;
        }
        diesel::sql_query(trimmed)
            .execute(&mut conn)
            .unwrap_or_else(|e| panic!("schema stmt failed: {} | stmt={}", e, trimmed));
    }
    conn
}

fn seed_account(conn: &mut SqliteConnection, uid: i64, name: &str) {
    AccountHandler
        .insert(
            conn,
            &Account {
                uid,
                name: name.into(),
                password_hash: "x".into(),
                email: String::new(),
                created_at: 1,
            },
        )
        .expect("insert account");
}

#[test]
fn object_roundtrip_create_get_move_delete() {
    let mut conn = fresh_conn();
    let repo = ObjectRepo;

    let room = repo.create(&mut conn, "room", "Limbo", None).unwrap();
    assert_eq!(room.type_key, "room");
    assert_eq!(room.name, "Limbo");
    assert_eq!(room.location_uid, None);

    let item = repo
        .create(&mut conn, "item", "rock", Some(room.uid))
        .unwrap();
    assert_eq!(item.location_uid, Some(room.uid));

    let contents = repo.contents_of(&mut conn, room.uid).unwrap();
    assert_eq!(contents.len(), 1);
    assert_eq!(contents[0].uid, item.uid);

    repo.move_to(&mut conn, item.uid, None).unwrap();
    let after = repo.get(&mut conn, item.uid).unwrap().unwrap();
    assert_eq!(after.location_uid, None);

    let n = repo.delete(&mut conn, item.uid).unwrap();
    assert_eq!(n, 1);
    assert!(repo.get(&mut conn, item.uid).unwrap().is_none());
}

#[test]
fn attribute_roundtrip_set_get_all_delete() {
    let mut conn = fresh_conn();
    let obj_repo = ObjectRepo;
    let attr_repo = AttributeRepo;

    let mob = obj_repo
        .create(&mut conn, "mob", "goblin", None)
        .unwrap();

    attr_repo
        .set(&mut conn, mob.uid, "hp", &serde_json::json!(20))
        .unwrap();
    attr_repo
        .set(&mut conn, mob.uid, "loot", &serde_json::json!(["coin", "knife"]))
        .unwrap();

    let hp = attr_repo.get(&mut conn, mob.uid, "hp").unwrap().unwrap();
    assert_eq!(hp, serde_json::json!(20));

    let all = attr_repo.all(&mut conn, mob.uid).unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all["loot"], serde_json::json!(["coin", "knife"]));

    // upsert overwrites
    attr_repo
        .set(&mut conn, mob.uid, "hp", &serde_json::json!(15))
        .unwrap();
    let hp = attr_repo.get(&mut conn, mob.uid, "hp").unwrap().unwrap();
    assert_eq!(hp, serde_json::json!(15));

    let n = attr_repo.delete(&mut conn, mob.uid, "hp").unwrap();
    assert_eq!(n, 1);
    assert!(attr_repo.get(&mut conn, mob.uid, "hp").unwrap().is_none());
}

#[test]
fn tag_roundtrip_add_idempotent_lookup_remove() {
    let mut conn = fresh_conn();
    let obj_repo = ObjectRepo;
    let tag_repo = TagRepo;

    let r1 = obj_repo.create(&mut conn, "room", "Hall", None).unwrap();
    let r2 = obj_repo.create(&mut conn, "room", "Cellar", None).unwrap();

    tag_repo.add(&mut conn, r1.uid, "safe-zone", "perm").unwrap();
    tag_repo.add(&mut conn, r2.uid, "safe-zone", "perm").unwrap();
    tag_repo.add(&mut conn, r1.uid, "safe-zone", "perm").unwrap(); // idempotent

    assert!(tag_repo.has(&mut conn, r1.uid, "safe-zone", "perm").unwrap());
    let mut hits = tag_repo.objects_with(&mut conn, "safe-zone", "perm").unwrap();
    hits.sort();
    assert_eq!(hits.len(), 2);

    tag_repo.remove(&mut conn, r1.uid, "safe-zone", "perm").unwrap();
    assert!(!tag_repo.has(&mut conn, r1.uid, "safe-zone", "perm").unwrap());
}

#[test]
fn character_account_link_unlink_list() {
    let mut conn = fresh_conn();
    let obj_repo = ObjectRepo;
    let link_repo = CharacterAccountRepo;

    seed_account(&mut conn, 1, "alice");

    let c1 = obj_repo.create(&mut conn, "character", "Alice", None).unwrap();
    let c2 = obj_repo.create(&mut conn, "character", "Alex", None).unwrap();

    let l1 = link_repo.link(&mut conn, c1.uid, 1).unwrap();
    assert_eq!(l1.ordinal, 0);
    let l2 = link_repo.link(&mut conn, c2.uid, 1).unwrap();
    assert_eq!(l2.ordinal, 1);

    let listed = link_repo.list_for_account(&mut conn, 1).unwrap();
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].object_uid, c1.uid);

    link_repo.unlink(&mut conn, c1.uid).unwrap();
    let listed = link_repo.list_for_account(&mut conn, 1).unwrap();
    assert_eq!(listed.len(), 1);
}
```

- [ ] **Step 2: Run the tests**

```bash
cd /home/duys/.repos/MuOxi && cargo test -p db --features db-sqlite 2>&1 | tail -20
```

Expected: 5 tests pass — the existing `account_roundtrip` from Plan 1 plus the four new ones.

- [ ] **Step 3: Commit**

```bash
git add db/tests/integration_objects.rs
git commit -m "test(db): roundtrip tests for object/attribute/tag/character_account"
```

---

## Task 11: Update root + db AGENTS.md

**Files:**
- Modify: `AGENTS.md`
- Modify: `db/AGENTS.md`

- [ ] **Step 1: Root AGENTS.md**

In `/home/duys/.repos/MuOxi/AGENTS.md`:

- WHERE TO LOOK: change `Database tables / Diesel ORM` row to point at `db/src/objects/` and add a row `Generic in-world objects (rooms/items/mobs/characters)` → `db/src/objects/`.
- CODE MAP: add rows for `Object`, `ObjectRepo`, `AttributeRepo`, `TagRepo`, `CharacterAccountRepo` (location: `db/src/objects/`).
- ANTI-PATTERNS: add: **DO NOT add a separate `characters` table.** Characters are objects with `type_key = "character"`.

- [ ] **Step 2: db/AGENTS.md**

In `/home/duys/.repos/MuOxi/db/AGENTS.md`:

- STRUCTURE: add the `objects/` directory with its four files.
- CORE TYPES: add `Object`, `ObjectAttribute`, `ObjectTag`, `CharacterAccount`, `ObjectRepo`, `AttributeRepo`, `TagRepo`, `CharacterAccountRepo` rows.
- DATABASE SCHEMA: replace the existing schema block with the post-Task 1 schema (objects, object_attributes, object_tags, character_accounts, plus accounts).
- ADDING A NEW IN-WORLD TYPE: new section explaining "register a TypeClass with the engine (Plan 4) — schema does not need to change."
- ANTI-PATTERNS: add: **DO NOT bypass the repos.** Engine code calls `db.objects.create(...)`, never `diesel::insert_into(objects::table)` directly.

- [ ] **Step 3: Commit**

```bash
git add AGENTS.md db/AGENTS.md
git commit -m "docs(agents): document objects/attributes/tags model"
```

---

## Verification Summary

A successful run of this plan ends with:

- [ ] `migrations/2026-05-07-000100_objects/{up,down}.sql` exist and parse on SQLite.
- [ ] `db/src/objects/` contains `mod.rs`, `object.rs`, `attribute.rs`, `tag.rs`, `character_account.rs`.
- [ ] `cargo check -p db` and `cargo test -p db --features db-sqlite` both succeed.
- [ ] All 5 db integration tests (1 from Plan 1 + 4 new) pass.
- [ ] `DatabaseHandler` exposes `objects`, `attributes`, `tags`, `character_accounts` fields.
- [ ] No `characters` or `account_characters` tables remain in `schema.rs`.
- [ ] Engine code (`muoxi/src/server/`) was NOT changed in this plan — Plan 6 wires the auth flow into the new model.
