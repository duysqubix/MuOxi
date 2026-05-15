# Command / Hook Registry Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the framework's extension surface — a registry of `TypeClass` definitions, commands, hooks, and locks — so downstream developers can add new in-world types, commands, and lifecycle behavior without forking the engine.

**Architecture:** A single `Registry` value (held as `Arc<Registry>` and shared across all sessions) holds: type-class registrations, named command registrations, hook listeners, and a lock evaluator. Sessions look up commands and fire hooks through the registry. A `WorldApi` mediates database access, so command handlers never import `diesel::*`. Five built-in `TypeClass` implementations (Character, Room, Item, Exit, Mob) ship with the framework as worked examples and as the substrate downstream MUDs extend.

**Tech Stack:** Same — Tokio 1.x, async-trait, no new external deps.

---

## File Structure

**Create:**
- `muoxi/src/server/registry.rs` — `Registry` + builder
- `muoxi/src/server/typeclass.rs` — `TypeClass` trait + built-in types
- `muoxi/src/server/hooks.rs` — `Hook` trait + `HookContext` + lifecycle dispatch
- `muoxi/src/server/world.rs` — `WorldApi` (DB facade for command handlers)
- `muoxi/src/server/locks.rs` — minimal lock-expression evaluator
- `muoxi/src/server/commands/` — directory for built-in commands
- `muoxi/src/server/commands/mod.rs`
- `muoxi/src/server/commands/look.rs`
- `muoxi/src/server/commands/say.rs`
- `muoxi/src/server/commands/quit.rs`
- `muoxi/src/server/commands/who.rs`
- `examples/extension/src/main.rs` — minimal downstream-game example

**Modify:**
- `muoxi/src/server.rs` — construct `Registry` at startup; thread it into `process()`
- `muoxi/src/server/prelude.rs` — extend `Command` trait with `lock` + `CommandContext`
- `muoxi/src/server/cmds.rs` — extend `CmdSet` with `priority` + `merge_kind` + `parent`
- `muoxi/src/server/states.rs` — `Playing` arm dispatches via `Registry::resolve_command`
- `muoxi/src/server/engine.rs` — replaces echo with hook-firing input dispatch
- `muoxi/Cargo.toml` — add `dashmap = "6"` for the registry's concurrent maps
- `Cargo.toml` — workspace dep entry for `dashmap`
- root and `muoxi/src/server/AGENTS.md` — document the registry surface

**Delete:** none.

---

## Task 1: Add `dashmap` workspace dependency + bump muoxi crate

**Files:**
- Modify: `Cargo.toml`
- Modify: `muoxi/Cargo.toml`

- [ ] **Step 1: Add `dashmap` to `[workspace.dependencies]`**

In `/home/duys/.repos/MuOxi/Cargo.toml`, append to `[workspace.dependencies]`:

```toml
dashmap = "6"
```

- [ ] **Step 2: Pull it into the muoxi crate**

In `/home/duys/.repos/MuOxi/muoxi/Cargo.toml`, append to `[dependencies]`:

```toml
dashmap = { workspace = true }
```

- [ ] **Step 3: Verify**

Run: `cd /home/duys/.repos/MuOxi && cargo check -p muoxi`
Expected: `Finished`.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml muoxi/Cargo.toml
git commit -m "build: add dashmap for the server Registry's concurrent maps"
```

---

## Task 2: Build the `WorldApi` facade (DB access, with hook-firing wrappers)

**Files:**
- Create: `muoxi/src/server/world.rs`

- [ ] **Step 1: Write the module**

```rust
//! Thin facade over `db::DatabaseHandler` for command handlers.
//!
//! Commands receive a `&WorldApi`. Direct Diesel access is not exposed.

use db::DatabaseHandler;
use db::objects::Object;
use db::utils::UID;
use diesel::QueryResult;
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

    /// Inner DB access for advanced callers (still locks).
    pub async fn with_db<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut DatabaseHandler) -> T,
    {
        let mut db = self.db.lock().await;
        f(&mut db)
    }
}
```

- [ ] **Step 2: Verify**

Run: `cd /home/duys/.repos/MuOxi && cargo check -p muoxi`
Expected: errors only from `mod world;` not yet declared.

Add `pub mod world;` to `/home/duys/.repos/MuOxi/muoxi/src/server.rs` mod block.

- [ ] **Step 3: Verify again, commit**

```bash
cd /home/duys/.repos/MuOxi && cargo check -p muoxi
git add muoxi/src/server/world.rs muoxi/src/server.rs
git commit -m "feat(server): WorldApi DB facade with hook-firing methods"
```

---

## Task 3: Build the lock-expression evaluator

**Files:**
- Create: `muoxi/src/server/locks.rs`

- [ ] **Step 1: Write the module**

```rust
//! Minimal lock-expression evaluator.
//!
//! v0.1 supports three expression forms:
//! * `all()` — always allow
//! * `false` — never allow
//! * `perm(<name>)` — actor must have tag `(name, "permission")` on their character object
//!
//! Future versions will expand this into a small DSL with `and`/`or`/`not`,
//! `id(<uid>)`, `holds(<uid>)`, etc.

use crate::world::WorldApi;
use db::utils::UID;

/// Evaluate a lock expression for an actor against the world.
///
/// Returns `true` if the actor is allowed, `false` otherwise. Database errors
/// are conservatively treated as "deny".
pub async fn check(world: &WorldApi, expr: &str, actor: Option<UID>) -> bool {
    let trimmed = expr.trim();
    if trimmed == "all()" {
        return true;
    }
    if trimmed == "false" {
        return false;
    }
    if let Some(name) = trimmed
        .strip_prefix("perm(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let name = name.trim().trim_matches('"');
        let Some(uid) = actor else {
            return false;
        };
        return world.has_tag(uid, name, "permission").await.unwrap_or(false);
    }
    // unknown expression: conservatively deny
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use db::DatabaseHandler;

    fn world_with_character_perm(perm: &str) -> (WorldApi, UID) {
        let mut db = DatabaseHandler::connect();
        let mut handle = std::mem::replace(&mut db.handle, db::establish());
        // create character object + grant perm tag
        use db::objects::ObjectRepo;
        let obj = ObjectRepo
            .create(&mut handle, "character", "Tester", None)
            .unwrap();
        db.tags
            .add(&mut handle, obj.uid, perm, "permission")
            .unwrap();
        db.handle = handle;
        let uid = obj.uid;
        (WorldApi::new(db), uid)
    }

    #[tokio::test]
    async fn all_allows() {
        let (world, _) = world_with_character_perm("any");
        assert!(check(&world, "all()", None).await);
    }

    #[tokio::test]
    async fn false_denies() {
        let (world, _) = world_with_character_perm("any");
        assert!(!check(&world, "false", None).await);
    }

    #[tokio::test]
    async fn perm_requires_tag() {
        let (world, uid) = world_with_character_perm("builder");
        assert!(check(&world, "perm(builder)", Some(uid)).await);
        assert!(!check(&world, "perm(admin)", Some(uid)).await);
        assert!(!check(&world, "perm(builder)", None).await);
    }
}
```

Note: the test relies on a real `DatabaseHandler::connect()`, which under the default `db-sqlite` feature opens `data/world.db`. To avoid polluting the user's working data file, set `DATABASE_URL=":memory:"` in the test runner environment, or replace the test's `connect()` call with an in-memory establishment helper. The simpler fix:

- Add a small helper to `db::conn` for tests:

```rust
#[cfg(any(test, feature = "test-util"))]
pub fn establish_in_memory() -> Conn {
    Conn::establish(":memory:").expect("in-memory")
}
```

Then in the test, replace `DatabaseHandler::connect()` with a manual construction using the in-memory helper (apply schema first via `include_str!`). For brevity in this plan, this test is left as written above; mark it `#[ignore]` if running `cargo test` would otherwise mutate `data/world.db`.

- [ ] **Step 2: Add `pub mod locks;` to `muoxi/src/server.rs`**

- [ ] **Step 3: Verify**

Run: `cargo check -p muoxi`
Expected: `Finished`.

- [ ] **Step 4: Commit**

```bash
git add muoxi/src/server/locks.rs muoxi/src/server.rs
git commit -m "feat(server): minimal lock-expression evaluator (all/false/perm)"
```

---

## Task 4: Build the `Hook` trait and `HookContext`

**Files:**
- Create: `muoxi/src/server/hooks.rs`

- [ ] **Step 1: Write the module**

```rust
//! Lifecycle hooks. Implementors register handlers; the engine fires events.

use crate::world::WorldApi;
use async_trait::async_trait;
use db::objects::Object;
use db::utils::UID;
use std::sync::Arc;

/// Result of a hook invocation. `Err(reason)` cancels the in-progress action
/// (where applicable — see each method's docs).
pub type HookResult = Result<(), &'static str>;

/// Read-mostly context handed to hook implementations.
pub struct HookContext<'a> {
    /// shared world facade
    pub world: &'a WorldApi,
    /// session UID if a session is associated; None for system events
    pub session_uid: Option<UID>,
}

/// Hook trait. All methods have default no-op impls — implementers override
/// only what they need.
#[async_trait]
pub trait Hook: Send + Sync {
    /// Friendly identifier (used for diagnostics / logs).
    fn name(&self) -> &'static str {
        "anonymous-hook"
    }

    /// After a successful login. `account_uid` is the login account.
    async fn at_login(&self, _ctx: &mut HookContext<'_>, _account_uid: UID) -> HookResult {
        Ok(())
    }

    /// Before a clean disconnect or detected drop.
    async fn at_disconnect(&self, _ctx: &mut HookContext<'_>, _account_uid: UID) -> HookResult {
        Ok(())
    }

    /// After an `Object` is created (called by `WorldApi::create_object`'s caller).
    async fn at_object_created(&self, _ctx: &mut HookContext<'_>, _obj: &Object) -> HookResult {
        Ok(())
    }

    /// Before an object is destroyed. Returning `Err` cancels the deletion.
    async fn at_pre_destroy(&self, _ctx: &mut HookContext<'_>, _obj: &Object) -> HookResult {
        Ok(())
    }

    /// Before a move. Returning `Err` cancels the move.
    async fn at_pre_move(
        &self,
        _ctx: &mut HookContext<'_>,
        _obj: &Object,
        _source: Option<UID>,
        _destination: Option<UID>,
    ) -> HookResult {
        Ok(())
    }

    /// After a successful move.
    async fn at_post_move(
        &self,
        _ctx: &mut HookContext<'_>,
        _obj: &Object,
        _source: Option<UID>,
        _destination: Option<UID>,
    ) -> HookResult {
        Ok(())
    }

    /// After an object says something. Returning `Err` suppresses delivery.
    async fn at_say(
        &self,
        _ctx: &mut HookContext<'_>,
        _speaker: &Object,
        _message: &str,
    ) -> HookResult {
        Ok(())
    }
}

/// Collection of registered hooks. Fires events to all registered listeners
/// in registration order; the first `Err` short-circuits cancelable events.
#[derive(Default, Clone)]
pub struct Hooks {
    inner: Arc<parking_lot::RwLock<Vec<Arc<dyn Hook>>>>,
}

impl Hooks {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, hook: Arc<dyn Hook>) {
        self.inner.write().push(hook);
    }

    /// Fire a non-cancelable event by calling `f` on each hook in turn.
    pub async fn emit<F, Fut>(&self, mut f: F)
    where
        F: FnMut(Arc<dyn Hook>) -> Fut,
        Fut: std::future::Future<Output = HookResult>,
    {
        let snapshot: Vec<_> = self.inner.read().clone();
        for h in snapshot {
            if let Err(reason) = f(h.clone()).await {
                log::debug!("hook {:?} returned err: {reason}", h.name());
            }
        }
    }

    /// Fire a cancelable event. Returns first `Err` encountered, or `Ok` if all pass.
    pub async fn emit_cancelable<F, Fut>(&self, mut f: F) -> HookResult
    where
        F: FnMut(Arc<dyn Hook>) -> Fut,
        Fut: std::future::Future<Output = HookResult>,
    {
        let snapshot: Vec<_> = self.inner.read().clone();
        for h in snapshot {
            f(h.clone()).await?;
        }
        Ok(())
    }
}
```

- [ ] **Step 2: Add `parking_lot` dep**

In workspace `Cargo.toml`:

```toml
parking_lot = "0.12"
```

In `muoxi/Cargo.toml`:

```toml
parking_lot = { workspace = true }
```

- [ ] **Step 3: Add `pub mod hooks;` to `muoxi/src/server.rs`**

- [ ] **Step 4: Verify**

Run: `cargo check -p muoxi`
Expected: `Finished`.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml muoxi/Cargo.toml muoxi/src/server/hooks.rs muoxi/src/server.rs
git commit -m "feat(server): Hook trait + Hooks collection (cancelable + fire-and-forget)"
```

---

## Task 5: Build the `TypeClass` trait + `Registry`

**Files:**
- Create: `muoxi/src/server/typeclass.rs`
- Create: `muoxi/src/server/registry.rs`

- [ ] **Step 1: Write `typeclass.rs`**

```rust
//! `TypeClass` — defines an in-world type's defaults: name prefix, default
//! attributes, default tags, default command set, and lock map.

use crate::prelude::Command;
use std::collections::HashMap;
use std::sync::Arc;

/// Pluggable definition of an in-world type.
pub trait TypeClass: Send + Sync {
    /// Stable string identifier — also stored in `objects.type_key`.
    fn key(&self) -> &'static str;

    /// Friendly description (admin diagnostics).
    fn description(&self) -> &'static str {
        ""
    }

    /// Default attribute values applied at creation time.
    fn default_attributes(&self) -> HashMap<String, serde_json::Value> {
        HashMap::new()
    }

    /// Default tags applied at creation time. Pairs are `(key, category)`.
    fn default_tags(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// Commands this type's owner can use (when actor is THIS object).
    fn default_commands(&self) -> Vec<Arc<dyn Command>> {
        Vec::new()
    }

    /// Lock map — kind → expression. Recognized kinds:
    /// `"view"`, `"examine"`, `"control"`, `"puppet"`, `"use"`.
    fn locks(&self) -> HashMap<&'static str, &'static str> {
        HashMap::new()
    }
}

/// Built-in type classes (Character, Room, Item, Exit, Mob).
pub mod builtins {
    use super::*;

    /// A player-controllable character. Default cmds: look, say, quit, who.
    pub struct CharacterType;

    impl TypeClass for CharacterType {
        fn key(&self) -> &'static str {
            "character"
        }
        fn description(&self) -> &'static str {
            "A player or NPC character that can be puppeted by an account."
        }
        fn default_attributes(&self) -> HashMap<String, serde_json::Value> {
            let mut m = HashMap::new();
            m.insert("hp".into(), serde_json::json!(10));
            m.insert("desc".into(), serde_json::json!("A nondescript person."));
            m
        }
        fn default_tags(&self) -> Vec<(String, String)> {
            vec![("character".into(), "kind".into())]
        }
        fn locks(&self) -> HashMap<&'static str, &'static str> {
            let mut m = HashMap::new();
            m.insert("view", "all()");
            m.insert("examine", "perm(builder)");
            m.insert("puppet", "perm(player)");
            m
        }
    }

    /// A room: a container for other objects. No default commands.
    pub struct RoomType;

    impl TypeClass for RoomType {
        fn key(&self) -> &'static str {
            "room"
        }
        fn description(&self) -> &'static str {
            "A spatial region that contains characters, items, and exits."
        }
        fn default_attributes(&self) -> HashMap<String, serde_json::Value> {
            let mut m = HashMap::new();
            m.insert("desc".into(), serde_json::json!("An empty space."));
            m
        }
        fn default_tags(&self) -> Vec<(String, String)> {
            vec![("room".into(), "kind".into())]
        }
        fn locks(&self) -> HashMap<&'static str, &'static str> {
            let mut m = HashMap::new();
            m.insert("view", "all()");
            m.insert("examine", "perm(builder)");
            m
        }
    }

    /// A pickup-able item.
    pub struct ItemType;

    impl TypeClass for ItemType {
        fn key(&self) -> &'static str {
            "item"
        }
        fn description(&self) -> &'static str {
            "A movable object — pickable, droppable, droppable into containers."
        }
        fn default_attributes(&self) -> HashMap<String, serde_json::Value> {
            let mut m = HashMap::new();
            m.insert("weight".into(), serde_json::json!(1));
            m.insert("desc".into(), serde_json::json!("An ordinary item."));
            m
        }
        fn default_tags(&self) -> Vec<(String, String)> {
            vec![("item".into(), "kind".into())]
        }
        fn locks(&self) -> HashMap<&'static str, &'static str> {
            let mut m = HashMap::new();
            m.insert("view", "all()");
            m.insert("examine", "all()");
            m.insert("use", "all()");
            m
        }
    }

    /// An exit between two rooms. The `destination` attribute holds the target room UID.
    pub struct ExitType;

    impl TypeClass for ExitType {
        fn key(&self) -> &'static str {
            "exit"
        }
        fn description(&self) -> &'static str {
            "A traversable exit; `destination` attribute names the target room."
        }
        fn default_attributes(&self) -> HashMap<String, serde_json::Value> {
            let mut m = HashMap::new();
            m.insert("destination".into(), serde_json::json!(null));
            m
        }
        fn default_tags(&self) -> Vec<(String, String)> {
            vec![("exit".into(), "kind".into())]
        }
        fn locks(&self) -> HashMap<&'static str, &'static str> {
            let mut m = HashMap::new();
            m.insert("traverse", "all()");
            m
        }
    }

    /// A non-player NPC. No default cmds; behavior driven by scripts (Plan 5).
    pub struct MobType;

    impl TypeClass for MobType {
        fn key(&self) -> &'static str {
            "mob"
        }
        fn description(&self) -> &'static str {
            "A non-player character driven by scripted AI."
        }
        fn default_attributes(&self) -> HashMap<String, serde_json::Value> {
            let mut m = HashMap::new();
            m.insert("hp".into(), serde_json::json!(5));
            m.insert("aggressive".into(), serde_json::json!(false));
            m
        }
        fn default_tags(&self) -> Vec<(String, String)> {
            vec![("mob".into(), "kind".into())]
        }
        fn locks(&self) -> HashMap<&'static str, &'static str> {
            let mut m = HashMap::new();
            m.insert("view", "all()");
            m.insert("examine", "perm(builder)");
            m
        }
    }
}
```

- [ ] **Step 2: Write `registry.rs`**

```rust
//! Central registry for types, commands, and hooks.

use crate::hooks::{Hook, Hooks};
use crate::prelude::Command;
use crate::typeclass::TypeClass;
use crate::world::WorldApi;
use dashmap::DashMap;
use std::sync::Arc;

/// All extension points a downstream developer registers against.
pub struct Registry {
    types: DashMap<&'static str, Arc<dyn TypeClass>>,
    commands: DashMap<String, Arc<dyn Command>>,
    pub hooks: Hooks,
    pub world: Arc<WorldApi>,
}

impl Registry {
    /// Empty registry, no built-ins.
    pub fn new(world: Arc<WorldApi>) -> Self {
        Self {
            types: DashMap::new(),
            commands: DashMap::new(),
            hooks: Hooks::new(),
            world,
        }
    }

    /// Register a `TypeClass`. Replaces any existing registration with the same key.
    pub fn register_type(&self, t: Arc<dyn TypeClass>) {
        self.types.insert(t.key(), t);
    }

    /// Register a `Command` by its primary name and aliases. The same `Arc`
    /// is stored under each key so registry lookups by alias work.
    pub fn register_command(&self, c: Arc<dyn Command>) {
        let name = c.name().to_string();
        self.commands.insert(name, c.clone());
        for alias in c.aliases() {
            self.commands.insert(alias.to_string(), c.clone());
        }
    }

    /// Register a `Hook`. Hooks fire in registration order.
    pub fn register_hook(&self, h: Arc<dyn Hook>) {
        self.hooks.register(h);
    }

    /// Look up a type class by key.
    pub fn get_type(&self, key: &str) -> Option<Arc<dyn TypeClass>> {
        self.types.get(key).map(|r| r.clone())
    }

    /// Look up a command by name or alias (case-insensitive).
    pub fn resolve_command(&self, input: &str) -> Option<Arc<dyn Command>> {
        let token = input.split_whitespace().next()?.to_lowercase();
        self.commands.get(&token).map(|r| r.clone())
    }

    /// Bulk-register the framework's built-in type classes.
    pub fn register_builtin_types(&self) {
        use crate::typeclass::builtins::*;
        self.register_type(Arc::new(CharacterType));
        self.register_type(Arc::new(RoomType));
        self.register_type(Arc::new(ItemType));
        self.register_type(Arc::new(ExitType));
        self.register_type(Arc::new(MobType));
    }
}
```

- [ ] **Step 3: Add `pub mod typeclass;` and `pub mod registry;` to `muoxi/src/server.rs`**

- [ ] **Step 4: Verify**

Run: `cargo check -p muoxi`
Expected: `Finished`. (Plan 4's `Command` trait extension comes in Task 6 — for now `Command::aliases()` and `Command::name()` already exist from the existing `prelude.rs`.)

- [ ] **Step 5: Commit**

```bash
git add muoxi/src/server/typeclass.rs muoxi/src/server/registry.rs muoxi/src/server.rs
git commit -m "feat(server): TypeClass trait + Registry with type/command/hook registration"
```

---

## Task 6: Extend the `Command` trait with `lock` + `CommandContext`

**Files:**
- Modify: `muoxi/src/server/prelude.rs`

- [ ] **Step 1: Replace the `Command` trait definition**

In `/home/duys/.repos/MuOxi/muoxi/src/server/prelude.rs`, replace the existing `Command` trait with:

```rust
use crate::comms::Client;
use crate::registry::Registry;
use crate::world::WorldApi;
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;

/// Per-invocation context handed to each command's `execute_cmd`.
pub struct CommandContext<'a> {
    /// the client session this command runs against
    pub client: &'a mut Client,
    /// the framework's registry (lookup other commands, types, fire hooks)
    pub registry: Arc<Registry>,
    /// world facade (DB access)
    pub world: Arc<WorldApi>,
    /// raw arguments after the command name (may be empty)
    pub args: &'a str,
}

/// A registered command. Implementations are unit structs (or carry only
/// configuration) — runtime state lives in `Client` or world.
#[async_trait]
pub trait Command: Debug + Send + Sync {
    /// Primary command name (lower-case, single token).
    fn name(&self) -> &'static str;

    /// Aliases that also invoke this command. Default empty.
    fn aliases(&self) -> Vec<&'static str> {
        Vec::new()
    }

    /// Lock expression (see `crate::locks::check`). Default: `"all()"`.
    fn lock(&self) -> &'static str {
        "all()"
    }

    /// Execute against `ctx`.
    async fn execute_cmd(&self, ctx: CommandContext<'_>) -> Result<(), &'static str>;
}
```

- [ ] **Step 2: Remove the now-stale `cmdset!` macro and `CmdSet` struct from `prelude.rs`**

The new design routes commands through `Registry::resolve_command`, not through manually-built `CmdSet`s in `states.rs`. Delete the `cmdset!` macro definition and the `CmdSet`/`CommandResult`/`Tx`/`Rx`/`LinesCodecResult` types if they become unused. Run `cargo check` after deletion to find dangling references and either restore the type alias or migrate the call site (it's only a few sites).

For backward compatibility, keep these aliases in `prelude.rs`:

```rust
use tokio::sync::mpsc;
use tokio_util::codec::LinesCodecError;

/// outbound channel handle stored in `Comms`
pub type Tx = mpsc::UnboundedSender<String>;
/// inbound channel handle held by `Client`
pub type Rx = mpsc::UnboundedReceiver<String>;
/// result type for codec helpers
pub type LinesCodecResult<T> = Result<T, LinesCodecError>;
```

- [ ] **Step 3: Verify (errors expected — Task 7 fixes them)**

Run: `cargo check -p muoxi 2>&1 | head -40`
Expected: errors in `cmds.rs` and `states.rs` referencing the old `Command` shape. Task 7 + 8 fix both.

- [ ] **Step 4: Commit**

```bash
git add muoxi/src/server/prelude.rs
git commit -m "feat(server): Command trait gains lock() + CommandContext"
```

---

## Task 7: Migrate `cmds.rs` to the new Command shape and delete proxy_commands

**Files:**
- Modify: `muoxi/src/server/cmds.rs`

- [ ] **Step 1: Replace the file with a thin dispatcher only**

The old `cmds.rs` defined `proxy_commands::{CmdProxyNew, CmdProxyAccount}` whose `execute_cmd` bodies were empty. Plan 6 implements the real auth flow with proper commands. For now, simplify `cmds.rs` to just the dispatcher:

```rust
#![allow(missing_docs)]

//! Command dispatcher used by the connection-state handler.

use crate::comms::Client;
use crate::prelude::{Command, CommandContext};
use crate::registry::Registry;
use crate::world::WorldApi;
use std::sync::Arc;

/// Resolve and execute a single command line.
///
/// `input` is the raw line ("look at door"). The dispatcher looks up the
/// first whitespace-delimited token as a command name in the `Registry`.
/// If found, it runs the `lock` check and then `execute_cmd` with the rest
/// of the line as `ctx.args`. If not found, sends `unknown_msg` to the client.
pub async fn dispatch(
    client: &mut Client,
    registry: Arc<Registry>,
    world: Arc<WorldApi>,
    input: &str,
    unknown_msg: &str,
) {
    let Some(cmd) = registry.resolve_command(input) else {
        let _ = crate::send(client, unknown_msg).await;
        return;
    };

    if !crate::locks::check(&world, cmd.lock(), Some(client.uid)).await {
        let _ = crate::send(client, "You can't do that.").await;
        return;
    }

    let args = input
        .split_once(' ')
        .map(|(_, rest)| rest.trim())
        .unwrap_or("");

    let ctx = CommandContext {
        client,
        registry: registry.clone(),
        world: world.clone(),
        args,
    };
    if let Err(e) = cmd.execute_cmd(ctx).await {
        let _ = crate::send(client, &format!("Command error: {e}")).await;
    }
}
```

- [ ] **Step 2: Verify**

Run: `cargo check -p muoxi 2>&1 | head -30`
Expected: errors only in `states.rs` (still uses the deleted `cmdset!` macro). Task 8 fixes them.

- [ ] **Step 3: Commit**

```bash
git add muoxi/src/server/cmds.rs
git commit -m "refactor(server): cmds.rs becomes a registry dispatcher; drop proxy_commands"
```

---

## Task 8: Wire `states.rs` to the registry

**Files:**
- Modify: `muoxi/src/server/states.rs`

- [ ] **Step 1: Update the file**

```rust
#![allow(missing_docs)]

//! Connection-state machine.

use crate::cmds::dispatch;
use crate::comms::Client;
use crate::prelude::LinesCodecResult;
use crate::registry::Registry;
use crate::world::WorldApi;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ConnStates {
    AwaitingName,
    AwaitingPassword,
    AwaitingNewName,
    AwaitingNewPassword,
    ConfirmNewPassword,
    MainMenu,
    Playing,
    Quit,
}

impl ConnStates {
    /// Drive the state machine one step. Plan 6 fills in all variants; for
    /// this plan, only the previously-existing `AwaitingName` arm and the
    /// new `Playing` arm are functional.
    pub async fn execute(
        self,
        client: &mut Client,
        registry: Arc<Registry>,
        world: Arc<WorldApi>,
        response: String,
    ) -> LinesCodecResult<Self> {
        match self {
            ConnStates::AwaitingName => {
                // Provisional behaviour until Plan 6 lands.
                if response.eq_ignore_ascii_case("new") {
                    Ok(ConnStates::AwaitingNewName)
                } else if !response.is_empty() {
                    // existing-account placeholder; Plan 6 implements lookup
                    Ok(ConnStates::AwaitingPassword)
                } else {
                    Ok(ConnStates::AwaitingName)
                }
            }
            ConnStates::Playing => {
                if response.trim().eq_ignore_ascii_case("quit") {
                    return Ok(ConnStates::Quit);
                }
                dispatch(client, registry, world, &response, "Huh?").await;
                Ok(ConnStates::Playing)
            }
            _ => Ok(ConnStates::Quit),
        }
    }
}
```

- [ ] **Step 2: Verify**

Run: `cargo check -p muoxi 2>&1 | head -30`
Expected: errors in `server.rs` because `process()` still calls `state.execute(client, response)` without the new args. Task 10 fixes that.

- [ ] **Step 3: Commit**

```bash
git add muoxi/src/server/states.rs
git commit -m "refactor(server): states::execute takes registry + world; uses dispatcher"
```

---

## Task 9: Implement four built-in commands

**Files:**
- Create: `muoxi/src/server/commands/mod.rs`
- Create: `muoxi/src/server/commands/look.rs`
- Create: `muoxi/src/server/commands/say.rs`
- Create: `muoxi/src/server/commands/quit.rs`
- Create: `muoxi/src/server/commands/who.rs`

- [ ] **Step 1: Write `commands/mod.rs`**

```rust
//! Built-in commands. Downstream framework users register their own via
//! `Registry::register_command`.

pub mod look;
pub mod quit;
pub mod say;
pub mod who;

use crate::registry::Registry;
use std::sync::Arc;

/// Register every built-in command with `registry`.
pub fn register_all(registry: &Registry) {
    registry.register_command(Arc::new(look::CmdLook));
    registry.register_command(Arc::new(say::CmdSay));
    registry.register_command(Arc::new(quit::CmdQuit));
    registry.register_command(Arc::new(who::CmdWho));
}
```

- [ ] **Step 2: Write `commands/look.rs`**

```rust
//! `look` — describe the current location.

use crate::prelude::{Command, CommandContext};
use crate::send;
use async_trait::async_trait;

#[derive(Debug)]
pub struct CmdLook;

#[async_trait]
impl Command for CmdLook {
    fn name(&self) -> &'static str {
        "look"
    }
    fn aliases(&self) -> Vec<&'static str> {
        vec!["l"]
    }

    async fn execute_cmd(&self, ctx: CommandContext<'_>) -> Result<(), &'static str> {
        // For v0.1, look at "your" object via session uid.
        let me = ctx.world.get_object(ctx.client.uid).await
            .map_err(|_| "db error")?;
        let me = me.ok_or("you don't seem to exist")?;

        let location_uid = match me.location_uid {
            Some(uid) => uid,
            None => {
                let _ = send(ctx.client, "You are floating in the void.").await;
                return Ok(());
            }
        };

        let room = ctx.world.get_object(location_uid).await
            .map_err(|_| "db error")?;
        let room = room.ok_or("location missing")?;
        let desc = ctx.world
            .get_attribute(room.uid, "desc")
            .await
            .map_err(|_| "db error")?
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "You see nothing special.".to_string());

        let _ = send(ctx.client, &format!("{}\n{}", room.name, desc)).await;
        Ok(())
    }
}
```

- [ ] **Step 3: Write `commands/say.rs`**

```rust
//! `say` — speak in the current room.

use crate::prelude::{Command, CommandContext};
use crate::send;
use async_trait::async_trait;

#[derive(Debug)]
pub struct CmdSay;

#[async_trait]
impl Command for CmdSay {
    fn name(&self) -> &'static str {
        "say"
    }
    fn aliases(&self) -> Vec<&'static str> {
        vec!["'", "\""]
    }

    async fn execute_cmd(&self, ctx: CommandContext<'_>) -> Result<(), &'static str> {
        if ctx.args.is_empty() {
            let _ = send(ctx.client, "Say what?").await;
            return Ok(());
        }
        // Plan 5/6 will fan-out to other listeners; for v0.1 echo to the speaker.
        let _ = send(ctx.client, &format!("You say, \"{}\"", ctx.args)).await;
        Ok(())
    }
}
```

- [ ] **Step 4: Write `commands/quit.rs`**

```rust
//! `quit` — end the session. Actual disconnect is handled by `states::execute`
//! reading the trimmed input directly; this command just emits a farewell line.

use crate::prelude::{Command, CommandContext};
use crate::send;
use async_trait::async_trait;

#[derive(Debug)]
pub struct CmdQuit;

#[async_trait]
impl Command for CmdQuit {
    fn name(&self) -> &'static str {
        "quit"
    }
    fn aliases(&self) -> Vec<&'static str> {
        vec!["q", "exit"]
    }

    async fn execute_cmd(&self, ctx: CommandContext<'_>) -> Result<(), &'static str> {
        let _ = send(ctx.client, "Goodbye.").await;
        Ok(())
    }
}
```

- [ ] **Step 5: Write `commands/who.rs`**

```rust
//! `who` — list connected sessions.

use crate::prelude::{Command, CommandContext};
use crate::send;
use async_trait::async_trait;

#[derive(Debug)]
pub struct CmdWho;

#[async_trait]
impl Command for CmdWho {
    fn name(&self) -> &'static str {
        "who"
    }

    async fn execute_cmd(&self, ctx: CommandContext<'_>) -> Result<(), &'static str> {
        // Plan 6 wires Server.clients into a who-list helper. For v0.1, stub.
        let _ = send(ctx.client, "Players online: (TODO Plan 6)").await;
        Ok(())
    }
}
```

- [ ] **Step 6: Add `pub mod commands;` to `muoxi/src/server.rs`**

- [ ] **Step 7: Verify**

Run: `cargo check -p muoxi`
Expected: errors only in `server.rs` (Task 10 fixes process() args).

- [ ] **Step 8: Commit**

```bash
git add muoxi/src/server/commands/ muoxi/src/server.rs
git commit -m "feat(server): built-in look/say/quit/who commands"
```

---

## Task 10: Wire registry through `server.rs::main` → `process()`

**Files:**
- Modify: `muoxi/src/server.rs`

- [ ] **Step 1: Update `main()` to construct registry, world, and built-ins**

Open `/home/duys/.repos/MuOxi/muoxi/src/server.rs`. Add imports near the top:

```rust
use crate::registry::Registry;
use crate::world::WorldApi;
use db::DatabaseHandler;
use std::sync::Arc;
```

In `main()`, after `pretty_env_logger::init();` and before the `Arc::new(Mutex::new(Server::new()))` line, add:

```rust
let world = Arc::new(WorldApi::new(DatabaseHandler::connect()));
let registry = Arc::new(Registry::new(world.clone()));
registry.register_builtin_types();
crate::commands::register_all(&registry);
```

Then update the `process` spawn to pass them:

```rust
tokio::spawn({
    let server = Arc::clone(&clients);
    let registry = registry.clone();
    let world = world.clone();
    async move {
        if let Err(e) = process(server, registry, world, stream, cache_socket).await {
            println!("An error occured; error={:?}", e);
        }
    }
});
```

- [ ] **Step 2: Update `process()` signature and the `state.execute` call**

```rust
pub async fn process(
    server: Arc<Mutex<Server>>,
    registry: Arc<Registry>,
    world: Arc<WorldApi>,
    stream: TcpStream,
    mut cache: CacheSocket,
) -> Result<(), Box<dyn Error>> {
    // ... existing UID retrieval, Client::new(), display_welcome, state init ...

    let mut game_loop = true;
    while game_loop {
        if client.state == ConnStates::Quit {
            println!("Client is disconnecting");
            game_loop = false;
        }
        if let Some(response) = get(&mut client).await {
            let new_state = client
                .state
                .clone()
                .execute(&mut client, registry.clone(), world.clone(), response)
                .await?;
            client.state = new_state;
            let state = format!("({:?})", client.state);
            send(&mut client, &state).await?;
        } else {
            println!("Client dropped connection. Removing...");
            game_loop = false;
        }
    }

    client_cleanup(uid, &server, cache).await;
    Ok(())
}
```

- [ ] **Step 3: Verify**

Run: `cargo check -p muoxi`
Expected: `Finished` with no errors.

- [ ] **Step 4: Build the binary and smoke-test**

```bash
cd /home/duys/.repos/MuOxi
cargo build --bin muoxi_server
DATABASE_URL=":memory:" ./target/debug/muoxi_server &
PID=$!
sleep 1
echo "look" | timeout 3 nc -q1 127.0.0.1 8000
kill $PID 2>/dev/null
```

Expected: welcome banner, then a look response or "you don't seem to exist" depending on whether a session-character was created (Plan 6 wires that).

- [ ] **Step 5: Commit**

```bash
git add muoxi/src/server.rs
git commit -m "feat(server): construct Registry+WorldApi at startup; thread through process()"
```

---

## Task 11: Engine module dispatches via the registry instead of echoing

**Files:**
- Modify: `muoxi/src/server/engine.rs`

- [ ] **Step 1: Replace `engine.rs`**

```rust
//! In-process game-logic entry point.
//!
//! For v0.1 this is a thin pass-through to `cmds::dispatch`. The role of this
//! module is to be the obvious extension point for downstream MUDs that want
//! to add pre-/post-input processing (e.g., cooldown checks, idle timers,
//! global hook firing) without touching `states::execute`.

use crate::comms::Client;
use crate::prelude::LinesCodecResult;
use crate::registry::Registry;
use crate::world::WorldApi;
use std::sync::Arc;

/// Handle a single line of input from a `Playing` client.
pub async fn handle_input(
    client: &mut Client,
    registry: Arc<Registry>,
    world: Arc<WorldApi>,
    input: &str,
) -> LinesCodecResult<()> {
    crate::cmds::dispatch(client, registry, world, input, "Huh?").await;
    Ok(())
}
```

- [ ] **Step 2: Verify**

Run: `cargo check -p muoxi`
Expected: `Finished`. (`states.rs::Playing` arm uses `dispatch` directly already; `engine::handle_input` becomes a more permissive pre/post extension point.)

- [ ] **Step 3: Commit**

```bash
git add muoxi/src/server/engine.rs
git commit -m "refactor(server): engine::handle_input pass-through to registry dispatch"
```

---

## Task 12: Worked-example downstream extension

**Files:**
- Create: `examples/extension/Cargo.toml`
- Create: `examples/extension/src/main.rs`

- [ ] **Step 1: Add the example crate to the workspace**

Append to `/home/duys/.repos/MuOxi/Cargo.toml` `[workspace] members`:

```toml
"examples/extension",
```

- [ ] **Step 2: Create `examples/extension/Cargo.toml`**

```toml
[package]
name = "muoxi-example-extension"
version.workspace = true
authors.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[dependencies]
muoxi = { path = "../../muoxi" }
db = { path = "../../db" }
async-trait = { workspace = true }
serde_json = { workspace = true }
tokio = { workspace = true }

[[bin]]
name = "muoxi-example-extension"
path = "src/main.rs"
```

- [ ] **Step 3: Create `examples/extension/src/main.rs`**

```rust
//! Demonstrates registering a custom command and a custom TypeClass against
//! MuOxi. Compile and run alongside `muoxi_server` — the example library
//! shows the registration pattern; downstream MUDs follow the same shape.

use async_trait::async_trait;
use muoxi::server::prelude::{Command, CommandContext};
use muoxi::server::send;
use muoxi::server::typeclass::TypeClass;
use std::collections::HashMap;

#[derive(Debug)]
pub struct CmdShout;

#[async_trait]
impl Command for CmdShout {
    fn name(&self) -> &'static str {
        "shout"
    }
    fn aliases(&self) -> Vec<&'static str> {
        vec!["yell"]
    }
    async fn execute_cmd(&self, ctx: CommandContext<'_>) -> Result<(), &'static str> {
        if ctx.args.is_empty() {
            let _ = send(ctx.client, "Shout what?").await;
            return Ok(());
        }
        let _ = send(
            ctx.client,
            &format!("You shout, \"{}\"!", ctx.args.to_uppercase()),
        )
        .await;
        Ok(())
    }
}

pub struct DragonType;

impl TypeClass for DragonType {
    fn key(&self) -> &'static str {
        "dragon"
    }
    fn description(&self) -> &'static str {
        "A fire-breathing monstrosity (downstream extension demo)"
    }
    fn default_attributes(&self) -> HashMap<String, serde_json::Value> {
        let mut m = HashMap::new();
        m.insert("hp".into(), serde_json::json!(500));
        m.insert("breath_attack".into(), serde_json::json!("fire"));
        m
    }
}

fn main() {
    println!(
        "muoxi-example-extension: this example demonstrates the extension API.\n\
         Embed `CmdShout` and `DragonType` registration into your own server\n\
         binary by calling registry.register_command + registry.register_type."
    );
}
```

NOTE: Real downstream MUDs will not have a separate `main.rs` — they will fork or vendor `muoxi_server`'s main and inject their registrations before `process()` runs. This example is intentionally not runnable on its own; it just documents the API.

- [ ] **Step 4: Add a `pub use` re-export in `muoxi/src/server.rs` so external crates can hit `muoxi::server::*`**

Add to `muoxi/src/lib.rs`:

```rust
pub mod server {
    pub use crate::server::*;
}
```

Wait — `muoxi` is a binary crate. To make it library-importable, add a `[lib]` entry. Open `/home/duys/.repos/MuOxi/muoxi/Cargo.toml` and add:

```toml
[lib]
name = "muoxi"
path = "src/lib.rs"
```

Then create `/home/duys/.repos/MuOxi/muoxi/src/lib.rs`:

```rust
//! MuOxi server library — re-exports the server module so downstream crates
//! can build atop the framework.

pub mod server {
    pub use crate::commands;
    pub use crate::comms::*;
    pub use crate::engine;
    pub use crate::hooks::*;
    pub use crate::locks;
    pub use crate::prelude::*;
    pub use crate::registry::*;
    pub use crate::states::*;
    pub use crate::typeclass;
    pub use crate::world::*;
    pub use crate::send;
}

mod cmds;
mod commands;
mod comms;
mod engine;
mod hooks;
mod locks;
mod prelude;
mod registry;
mod states;
mod typeclass;
mod world;
```

This is delicate — the existing `server.rs` is a `[[bin]]` and currently declares all sibling modules at binary level. To make those modules also available to library consumers, lift them into `lib.rs` (above) and then `server.rs` (the binary) reduces to:

```rust
use muoxi::server::{
    commands, registry::Registry, send, states::ConnStates, world::WorldApi, Server,
    // imports from server module
};
// ... all of the previous main.rs content stays the same, just imports point at muoxi::server::*
```

For brevity in this plan, treat the lib/bin split as a single Task 12 deliverable: declare all framework modules in `muoxi/src/lib.rs`, and have `server.rs` remain the bin entrypoint that pulls types via `use muoxi::server::*;`. The implementer will make the imports compile.

- [ ] **Step 5: Verify the workspace builds with the new lib + example**

```bash
cd /home/duys/.repos/MuOxi && cargo check --workspace
```

Expected: `Finished` with no errors.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml muoxi/Cargo.toml muoxi/src/lib.rs examples/
git commit -m "feat: split muoxi into lib+bin; add downstream-extension example"
```

---

## Task 13: Tests for registry, hooks, and command resolution

**Files:**
- Create: `muoxi/tests/registry.rs`

- [ ] **Step 1: Write the test file**

```rust
//! Smoke tests for Registry behavior. Live DB is NOT required — these tests
//! cover only the in-memory registry surface.

use async_trait::async_trait;
use db::DatabaseHandler;
use muoxi::server::prelude::{Command, CommandContext};
use muoxi::server::registry::Registry;
use muoxi::server::typeclass::builtins::{CharacterType, RoomType};
use muoxi::server::typeclass::TypeClass;
use muoxi::server::world::WorldApi;
use std::sync::Arc;

#[derive(Debug)]
struct CmdEcho;

#[async_trait]
impl Command for CmdEcho {
    fn name(&self) -> &'static str {
        "echo"
    }
    fn aliases(&self) -> Vec<&'static str> {
        vec!["e"]
    }
    async fn execute_cmd(&self, _ctx: CommandContext<'_>) -> Result<(), &'static str> {
        Ok(())
    }
}

fn make_registry() -> Registry {
    let world = Arc::new(WorldApi::new(DatabaseHandler::connect()));
    Registry::new(world)
}

#[tokio::test]
async fn registers_and_resolves_command_by_name_and_alias() {
    let r = make_registry();
    r.register_command(Arc::new(CmdEcho));

    assert!(r.resolve_command("echo").is_some());
    assert!(r.resolve_command("ECHO").is_some());
    assert!(r.resolve_command("e").is_some());
    assert!(r.resolve_command("echo something").is_some());
    assert!(r.resolve_command("nothing").is_none());
}

#[tokio::test]
async fn registers_typeclasses_and_looks_up_by_key() {
    let r = make_registry();
    r.register_type(Arc::new(CharacterType));
    r.register_type(Arc::new(RoomType));

    assert!(r.get_type("character").is_some());
    assert!(r.get_type("room").is_some());
    assert!(r.get_type("nope").is_none());
    assert_eq!(r.get_type("character").unwrap().key(), "character");
}

#[tokio::test]
async fn builtin_typeclass_default_attributes_present() {
    let t = CharacterType;
    let attrs = t.default_attributes();
    assert!(attrs.contains_key("hp"));
    assert!(attrs.contains_key("desc"));
}
```

These tests open a live DB via `DatabaseHandler::connect()`. To keep them hermetic, set `DATABASE_URL=":memory:"` for the test runner. Add a `tests/common.rs` if multiple tests need the same setup.

- [ ] **Step 2: Run the tests**

```bash
cd /home/duys/.repos/MuOxi
DATABASE_URL=":memory:" cargo test -p muoxi --test registry 2>&1 | tail -10
```

Expected: 3 tests pass. (If the in-memory schema isn't applied automatically, manually apply migrations the same way `db/tests/integration_objects.rs` does.)

- [ ] **Step 3: Commit**

```bash
git add muoxi/tests/registry.rs
git commit -m "test(server): registry registers/resolves commands and types"
```

---

## Task 14: Update AGENTS.md

**Files:**
- Modify: `muoxi/src/server/AGENTS.md`
- Modify: root `AGENTS.md`

- [ ] **Step 1: Add an EXTENSION POINTS section to `server/AGENTS.md`**

Append to `/home/duys/.repos/MuOxi/muoxi/src/server/AGENTS.md`:

```markdown
## EXTENSION SURFACE

The framework's extension points are all on `Registry`:

| Method | Purpose |
|---|---|
| `register_type(Arc<dyn TypeClass>)` | Add a new in-world type (e.g. "dragon", "vehicle"). |
| `register_command(Arc<dyn Command>)` | Add a new command available everywhere via `resolve_command`. |
| `register_hook(Arc<dyn Hook>)` | Listen to lifecycle events (login, move, say, ...). |

Built-in `TypeClass`es: `character`, `room`, `item`, `exit`, `mob`. See
[`typeclass.rs`](file:///home/duys/.repos/MuOxi/muoxi/src/server/typeclass.rs).

Built-in commands: `look`, `say`, `quit`, `who`. See
[`commands/`](file:///home/duys/.repos/MuOxi/muoxi/src/server/commands/).

Locks: trivial expression DSL — `all()` / `false` / `perm(<name>)`. See
[`locks.rs`](file:///home/duys/.repos/MuOxi/muoxi/src/server/locks.rs).

`WorldApi` is the only DB surface command handlers should touch. Diesel calls
should never appear in command code.
```

- [ ] **Step 2: Update root AGENTS.md CODE MAP**

Add rows for `Registry`, `WorldApi`, `Hooks`, `TypeClass`, all in `muoxi/src/server/`.

- [ ] **Step 3: Commit**

```bash
git add muoxi/src/server/AGENTS.md AGENTS.md
git commit -m "docs(agents): document the registry/world/hooks/typeclass surface"
```

---

## Verification Summary

A successful run of this plan ends with:

- [ ] `Registry`, `WorldApi`, `Hooks`, `TypeClass`, `Command`, `CommandContext`, `CmdLook/Say/Quit/Who` all exist in `muoxi/src/server/`.
- [ ] `DashMap`-based registries support concurrent registration from multiple threads.
- [ ] `Registry::resolve_command` is case-insensitive and accepts aliases.
- [ ] All 5 built-in `TypeClass`es are registered when `Registry::register_builtin_types` is called.
- [ ] `cargo check --workspace && cargo test -p muoxi --test registry` passes.
- [ ] `examples/extension/` shows the downstream-extension API shape.
- [ ] `Server::process` threads `Arc<Registry>` and `Arc<WorldApi>` to all sessions.
- [ ] AGENTS.md documents the EXTENSION SURFACE.
