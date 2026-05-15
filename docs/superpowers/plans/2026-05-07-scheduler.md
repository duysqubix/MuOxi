# Persistent Scheduler / Scripts Subsystem Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add Evennia-style persistent timed events ("scripts") so downstream MUDs can schedule heartbeats, ticks, mob AI updates, decay timers, respawns, weather rotations, channel broadcasts, and any other periodic or delayed behavior — surviving server restarts.

**Architecture:** A new `scripts` table records each scheduled job (handler key, interval, next-run timestamp, repeat-flag, persistent state, enabled-flag). A `Scheduler` runs as a background Tokio task, polls due jobs every 50ms, dispatches to a handler-key→handler registry, and persistently updates `next_run_at`. On boot, the scheduler scans for overdue jobs and runs them once before settling into the steady-state poll loop.

**Tech Stack:** Same — Tokio 1.x time facilities, no new external deps.

---

## File Structure

**Create:**
- `migrations/2026-05-07-000200_scripts/up.sql`
- `migrations/2026-05-07-000200_scripts/down.sql`
- `db/src/objects/script.rs` — `Script` model + `ScriptRepo`
- `muoxi/src/server/scheduler.rs` — `Scheduler` task + `ScriptHandler` trait + `ScriptContext`
- `muoxi/src/server/scripts/` — directory of built-in handlers
- `muoxi/src/server/scripts/mod.rs`
- `muoxi/src/server/scripts/heartbeat.rs` — example/demo handler
- `db/tests/integration_scripts.rs` — round-trip tests

**Modify:**
- `db/src/schema.rs` — add `scripts` table
- `db/src/objects/mod.rs` — re-export `Script` + `ScriptRepo`
- `db/src/lib.rs` — `DatabaseHandler` gains `scripts: ScriptRepo` field
- `muoxi/src/server/registry.rs` — `Registry` gains script-handler map + `register_script_handler` / `script_handler`
- `muoxi/src/server/world.rs` — `WorldApi` gains script CRUD wrappers
- `muoxi/src/server.rs` — spawn scheduler task at startup
- `muoxi/src/lib.rs` — re-export `scheduler`, `scripts`
- root and `db/`/`muoxi/` AGENTS.md — document the subsystem

**Delete:** none.

---

## Task 1: Write the migration

**Files:**
- Create: `migrations/2026-05-07-000200_scripts/up.sql`
- Create: `migrations/2026-05-07-000200_scripts/down.sql`

- [ ] **Step 1: Write `up.sql`**

```sql
-- scripts: persistent scheduled job records.
--
-- handler_key references a registered ScriptHandler at runtime. interval_ms
-- is how often the job repeats (0 means one-shot). next_run_at is unix
-- epoch milliseconds. state is per-job persistent JSON (serialized).
CREATE TABLE scripts (
    id           BIGINT  NOT NULL CHECK (id > 0),
    object_uid   BIGINT  NULL,
    handler_key  TEXT    NOT NULL,
    interval_ms  BIGINT  NOT NULL DEFAULT 0,
    next_run_at  BIGINT  NOT NULL,
    repeat       INTEGER NOT NULL DEFAULT 1,
    state        TEXT    NOT NULL DEFAULT '{}',
    enabled      INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (id),
    FOREIGN KEY (object_uid) REFERENCES objects(uid) ON DELETE CASCADE
);

CREATE INDEX idx_scripts_due ON scripts(enabled, next_run_at);
CREATE INDEX idx_scripts_handler ON scripts(handler_key);
CREATE INDEX idx_scripts_object ON scripts(object_uid);
```

- [ ] **Step 2: Write `down.sql`**

```sql
DROP TABLE IF EXISTS scripts;
```

- [ ] **Step 3: Verify against SQLite**

```bash
cd /home/duys/.repos/MuOxi
sqlite3 :memory: <<'EOF'
.read migrations/2026-05-07-000000_initial/up.sql
.read migrations/2026-05-07-000100_objects/up.sql
.read migrations/2026-05-07-000200_scripts/up.sql
.tables
EOF
```

Expected: `scripts` appears in the table list.

- [ ] **Step 4: Commit**

```bash
git add migrations/2026-05-07-000200_scripts/
git commit -m "feat(db): scripts table for persistent scheduled jobs"
```

---

## Task 2: Add `scripts` to `db/src/schema.rs`

**Files:**
- Modify: `db/src/schema.rs`

- [ ] **Step 1: Add the table macro**

Append to `/home/duys/.repos/MuOxi/db/src/schema.rs`:

```rust
diesel::table! {
    scripts (id) {
        id -> BigInt,
        object_uid -> Nullable<BigInt>,
        handler_key -> Text,
        interval_ms -> BigInt,
        next_run_at -> BigInt,
        repeat -> Integer,
        state -> Text,
        enabled -> Integer,
    }
}

diesel::joinable!(scripts -> objects (object_uid));
```

Update the `allow_tables_to_appear_in_same_query!` block to include `scripts`:

```rust
diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    objects,
    object_attributes,
    object_tags,
    character_accounts,
    scripts,
);
```

- [ ] **Step 2: Verify**

Run: `cd /home/duys/.repos/MuOxi && cargo check -p db`
Expected: `Finished`.

- [ ] **Step 3: Commit**

```bash
git add db/src/schema.rs
git commit -m "feat(db): schema entry for scripts table"
```

---

## Task 3: Implement `Script` model + `ScriptRepo`

**Files:**
- Create: `db/src/objects/script.rs`
- Modify: `db/src/objects/mod.rs`
- Modify: `db/src/lib.rs`

- [ ] **Step 1: Write `db/src/objects/script.rs`**

```rust
//! `Script` model and `ScriptRepo` — persistent scheduled jobs.

use crate::conn::Conn;
use crate::schema::scripts;
use crate::utils::{UID, gen_uid};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// A persistent scheduled job.
#[derive(Queryable, Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    /// unique id
    pub id: UID,
    /// owning object (None for global scripts)
    pub object_uid: Option<UID>,
    /// key registered with `Registry::register_script_handler`
    pub handler_key: String,
    /// re-fire interval in milliseconds (ignored if `repeat == 0`)
    pub interval_ms: i64,
    /// unix epoch milliseconds when the script should next execute
    pub next_run_at: i64,
    /// 1 = repeating, 0 = one-shot
    pub repeat: i32,
    /// per-script JSON-encoded persistent state
    pub state: String,
    /// 1 = enabled, 0 = disabled (won't be picked up)
    pub enabled: i32,
}

/// Insert payload for a new script.
#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = scripts)]
pub struct NewScript<'a> {
    /// pre-generated id
    pub id: UID,
    /// owning object uid
    pub object_uid: Option<UID>,
    /// handler key
    pub handler_key: &'a str,
    /// re-fire interval in ms
    pub interval_ms: i64,
    /// first-run timestamp (ms epoch)
    pub next_run_at: i64,
    /// 1=repeating, 0=one-shot
    pub repeat: i32,
    /// initial state JSON
    pub state: &'a str,
    /// enabled at creation (typically 1)
    pub enabled: i32,
}

/// CRUD for the `scripts` table.
pub struct ScriptRepo;

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

impl ScriptRepo {
    /// Create a one-shot script that runs `delay_ms` from now.
    pub fn create_oneshot(
        &self,
        conn: &mut Conn,
        object_uid: Option<UID>,
        handler_key: &str,
        delay_ms: i64,
        initial_state: &serde_json::Value,
    ) -> QueryResult<Script> {
        self.create(conn, object_uid, handler_key, delay_ms, 0, 0, initial_state)
    }

    /// Create a repeating script that fires every `interval_ms`. First run is
    /// `interval_ms` from now (set `delay_ms != interval_ms` for offset).
    pub fn create_repeating(
        &self,
        conn: &mut Conn,
        object_uid: Option<UID>,
        handler_key: &str,
        interval_ms: i64,
        initial_state: &serde_json::Value,
    ) -> QueryResult<Script> {
        self.create(
            conn,
            object_uid,
            handler_key,
            interval_ms,
            interval_ms,
            1,
            initial_state,
        )
    }

    fn create(
        &self,
        conn: &mut Conn,
        object_uid: Option<UID>,
        handler_key: &str,
        delay_ms: i64,
        interval_ms: i64,
        repeat: i32,
        initial_state: &serde_json::Value,
    ) -> QueryResult<Script> {
        let id = gen_uid();
        let next_run_at = now_ms() + delay_ms;
        let state_str = serde_json::to_string(initial_state)
            .map_err(|e| diesel::result::Error::SerializationError(Box::new(e)))?;
        let row = NewScript {
            id,
            object_uid,
            handler_key,
            interval_ms,
            next_run_at,
            repeat,
            state: &state_str,
            enabled: 1,
        };
        diesel::insert_into(scripts::table)
            .values(&row)
            .execute(conn)?;
        self.get(conn, id)?.ok_or(diesel::result::Error::NotFound)
    }

    /// Fetch by id.
    pub fn get(&self, conn: &mut Conn, id: UID) -> QueryResult<Option<Script>> {
        scripts::table
            .filter(scripts::id.eq(id))
            .first::<Script>(conn)
            .optional()
    }

    /// Delete a script.
    pub fn delete(&self, conn: &mut Conn, id: UID) -> QueryResult<usize> {
        diesel::delete(scripts::table.filter(scripts::id.eq(id))).execute(conn)
    }

    /// Disable (without deleting). Use when a handler returns Err.
    pub fn disable(&self, conn: &mut Conn, id: UID) -> QueryResult<usize> {
        diesel::update(scripts::table.filter(scripts::id.eq(id)))
            .set(scripts::enabled.eq(0))
            .execute(conn)
    }

    /// All enabled scripts whose `next_run_at` <= `now_ms()`.
    /// Returned in ascending `next_run_at` order so older-overdue runs first.
    pub fn list_due(&self, conn: &mut Conn) -> QueryResult<Vec<Script>> {
        let now = now_ms();
        scripts::table
            .filter(scripts::enabled.eq(1))
            .filter(scripts::next_run_at.le(now))
            .order(scripts::next_run_at.asc())
            .load::<Script>(conn)
    }

    /// After a successful run, advance `next_run_at` and persist new `state`.
    /// For one-shot scripts (`repeat == 0`), this disables the script.
    pub fn record_run(
        &self,
        conn: &mut Conn,
        id: UID,
        new_state: &serde_json::Value,
    ) -> QueryResult<usize> {
        let script = self
            .get(conn, id)?
            .ok_or(diesel::result::Error::NotFound)?;
        let state_str = serde_json::to_string(new_state)
            .map_err(|e| diesel::result::Error::SerializationError(Box::new(e)))?;
        if script.repeat == 0 {
            return diesel::update(scripts::table.filter(scripts::id.eq(id)))
                .set((scripts::enabled.eq(0), scripts::state.eq(state_str)))
                .execute(conn);
        }
        let next_run_at = now_ms() + script.interval_ms;
        diesel::update(scripts::table.filter(scripts::id.eq(id)))
            .set((
                scripts::next_run_at.eq(next_run_at),
                scripts::state.eq(state_str),
            ))
            .execute(conn)
    }
}
```

- [ ] **Step 2: Update `db/src/objects/mod.rs`**

Add `pub mod script;` and re-exports:

```rust
pub mod script;

pub use script::{NewScript, Script, ScriptRepo};
```

- [ ] **Step 3: Wire `scripts: ScriptRepo` into `DatabaseHandler`**

In `/home/duys/.repos/MuOxi/db/src/lib.rs`, add:

```rust
use objects::ScriptRepo;
```

And extend the struct + ctor:

```rust
pub struct DatabaseHandler {
    // ... existing fields ...
    /// scheduled-job CRUD
    pub scripts: ScriptRepo,
}

impl DatabaseHandler {
    pub fn connect() -> Self {
        // ...
        Self {
            // ... existing init ...
            scripts: ScriptRepo,
        }
    }
}
```

- [ ] **Step 4: Verify**

Run: `cargo check -p db`
Expected: `Finished`.

- [ ] **Step 5: Commit**

```bash
git add db/src/objects/script.rs db/src/objects/mod.rs db/src/lib.rs
git commit -m "feat(db): Script model + ScriptRepo with create/list_due/record_run"
```

---

## Task 4: Round-trip tests for `ScriptRepo`

**Files:**
- Create: `db/tests/integration_scripts.rs`

- [ ] **Step 1: Write the test**

```rust
//! Round-trip tests for ScriptRepo on in-memory SQLite.

#![cfg(feature = "db-sqlite")]

use db::objects::{ObjectRepo, ScriptRepo};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use std::time::{SystemTime, UNIX_EPOCH};

const SCHEMA_INITIAL: &str = include_str!("../../migrations/2026-05-07-000000_initial/up.sql");
const SCHEMA_OBJECTS: &str = include_str!("../../migrations/2026-05-07-000100_objects/up.sql");
const SCHEMA_SCRIPTS: &str = include_str!("../../migrations/2026-05-07-000200_scripts/up.sql");

fn fresh_conn() -> SqliteConnection {
    let mut conn = SqliteConnection::establish(":memory:").expect("memory sqlite");
    diesel::sql_query("PRAGMA foreign_keys = ON")
        .execute(&mut conn)
        .unwrap();
    for src in [SCHEMA_INITIAL, SCHEMA_OBJECTS, SCHEMA_SCRIPTS] {
        for stmt in src.split(';') {
            let trimmed = stmt.trim();
            if trimmed.is_empty() {
                continue;
            }
            diesel::sql_query(trimmed)
                .execute(&mut conn)
                .unwrap_or_else(|e| panic!("schema stmt failed: {}", e));
        }
    }
    conn
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

#[test]
fn oneshot_roundtrip_and_disable_after_run() {
    let mut conn = fresh_conn();
    let repo = ScriptRepo;
    let s = repo
        .create_oneshot(
            &mut conn,
            None,
            "demo",
            0,
            &serde_json::json!({"k": 1}),
        )
        .unwrap();
    assert_eq!(s.repeat, 0);
    assert_eq!(s.enabled, 1);

    // due immediately (delay 0)
    let due = repo.list_due(&mut conn).unwrap();
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].id, s.id);

    // simulate a successful run
    repo.record_run(&mut conn, s.id, &serde_json::json!({"k": 2}))
        .unwrap();
    let after = repo.get(&mut conn, s.id).unwrap().unwrap();
    assert_eq!(after.enabled, 0); // one-shot disables itself
    assert_eq!(after.state, "{\"k\":2}");

    let due_after = repo.list_due(&mut conn).unwrap();
    assert!(due_after.is_empty());
}

#[test]
fn repeating_advances_next_run_at() {
    let mut conn = fresh_conn();
    let repo = ScriptRepo;
    let s = repo
        .create_repeating(
            &mut conn,
            None,
            "tick",
            1000,
            &serde_json::json!({}),
        )
        .unwrap();
    let started_at = now_ms();
    assert!(s.next_run_at >= started_at + 999);

    // record a run -> next_run_at advances by interval_ms
    let pre_run_at_max = now_ms() + 1100;
    repo.record_run(&mut conn, s.id, &serde_json::json!({"ran": 1}))
        .unwrap();
    let after = repo.get(&mut conn, s.id).unwrap().unwrap();
    assert_eq!(after.enabled, 1);
    assert!(after.next_run_at >= pre_run_at_max - 200);
    assert!(after.next_run_at <= pre_run_at_max + 200);
}

#[test]
fn disable_removes_from_due_list() {
    let mut conn = fresh_conn();
    let repo = ScriptRepo;
    let s = repo
        .create_oneshot(&mut conn, None, "demo", 0, &serde_json::json!({}))
        .unwrap();
    assert_eq!(repo.list_due(&mut conn).unwrap().len(), 1);
    repo.disable(&mut conn, s.id).unwrap();
    assert!(repo.list_due(&mut conn).unwrap().is_empty());
}

#[test]
fn fk_cascade_deletes_object_scripts() {
    let mut conn = fresh_conn();
    let obj_repo = ObjectRepo;
    let script_repo = ScriptRepo;

    let mob = obj_repo.create(&mut conn, "mob", "goblin", None).unwrap();
    let s = script_repo
        .create_repeating(&mut conn, Some(mob.uid), "ai", 5000, &serde_json::json!({}))
        .unwrap();

    obj_repo.delete(&mut conn, mob.uid).unwrap();
    let after = script_repo.get(&mut conn, s.id).unwrap();
    assert!(after.is_none(), "ON DELETE CASCADE should drop the script");
}
```

- [ ] **Step 2: Run**

```bash
cd /home/duys/.repos/MuOxi && cargo test -p db --features db-sqlite --test integration_scripts 2>&1 | tail -10
```

Expected: 4 tests pass.

- [ ] **Step 3: Commit**

```bash
git add db/tests/integration_scripts.rs
git commit -m "test(db): ScriptRepo create/run/disable/cascade roundtrip"
```

---

## Task 5: Add script-handler registration to `Registry`

**Files:**
- Modify: `muoxi/src/server/registry.rs`

- [ ] **Step 1: Extend `Registry`**

Add a new field + methods to `/home/duys/.repos/MuOxi/muoxi/src/server/registry.rs`:

```rust
use crate::scheduler::ScriptHandler;

// inside the Registry struct:
script_handlers: DashMap<String, Arc<dyn ScriptHandler>>,
```

Initialize in `Registry::new`:

```rust
script_handlers: DashMap::new(),
```

Add methods:

```rust
/// Register a script handler. Replaces any previous handler with the same key.
pub fn register_script_handler(&self, h: Arc<dyn ScriptHandler>) {
    self.script_handlers.insert(h.key().to_string(), h);
}

/// Resolve a script handler by key.
pub fn script_handler(&self, key: &str) -> Option<Arc<dyn ScriptHandler>> {
    self.script_handlers.get(key).map(|r| r.clone())
}
```

- [ ] **Step 2: Verify (errors expected — Task 6 defines `ScriptHandler`)**

Run: `cargo check -p muoxi 2>&1 | head -10`
Expected: error "cannot find trait `ScriptHandler` in `crate::scheduler`" — that's the cue for Task 6.

- [ ] **Step 3: Commit**

```bash
git add muoxi/src/server/registry.rs
git commit -m "feat(server): Registry exposes script-handler registration"
```

---

## Task 6: Implement `Scheduler` + `ScriptHandler` trait

**Files:**
- Create: `muoxi/src/server/scheduler.rs`

- [ ] **Step 1: Write the module**

```rust
//! Persistent scheduler for `Script` rows.
//!
//! `Scheduler::run` is the long-lived background task. It polls the DB every
//! `POLL_INTERVAL` ms for due scripts, resolves their `handler_key` against
//! the `Registry`, and calls `ScriptHandler::run`. Successful runs advance
//! `next_run_at`; errors disable the script.

use crate::registry::Registry;
use crate::world::WorldApi;
use async_trait::async_trait;
use db::utils::UID;
use std::sync::Arc;
use std::time::Duration;

/// Per-run context passed to `ScriptHandler::run`.
pub struct ScriptContext<'a> {
    /// the world facade (DB access)
    pub world: &'a WorldApi,
    /// the registry (so handlers can look up other handlers, types, etc.)
    pub registry: Arc<Registry>,
}

/// Implement this for any periodic / scheduled behavior.
///
/// The framework provides one built-in handler (`HeartbeatHandler`) as a
/// worked example; downstream MUDs register their own.
#[async_trait]
pub trait ScriptHandler: Send + Sync {
    /// Stable identifier — also stored in `scripts.handler_key`.
    fn key(&self) -> &'static str;

    /// Execute one tick. Receive the script's persistent JSON state, return
    /// the new state to persist. Return `Err` to disable the script.
    async fn run(
        &self,
        ctx: &mut ScriptContext<'_>,
        object_uid: Option<UID>,
        state: serde_json::Value,
    ) -> Result<serde_json::Value, &'static str>;
}

/// How often the scheduler scans the DB for due jobs.
const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// The scheduler task. Spawn one of these per server instance.
pub struct Scheduler {
    registry: Arc<Registry>,
}

impl Scheduler {
    pub fn new(registry: Arc<Registry>) -> Self {
        Self { registry }
    }

    /// Run forever. Cancel by dropping the spawned `JoinHandle`.
    pub async fn run(self) {
        loop {
            if let Err(e) = self.tick().await {
                log::warn!("scheduler tick error: {e}");
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    }

    async fn tick(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let due = self
            .registry
            .world
            .with_db(|db| db.scripts.list_due(&mut db.handle))
            .await?;

        for script in due {
            let handler = match self.registry.script_handler(&script.handler_key) {
                Some(h) => h,
                None => {
                    log::warn!(
                        "script {} references unregistered handler {:?}; disabling",
                        script.id,
                        script.handler_key
                    );
                    self.registry
                        .world
                        .with_db(|db| db.scripts.disable(&mut db.handle, script.id))
                        .await
                        .ok();
                    continue;
                }
            };

            let parsed_state: serde_json::Value =
                serde_json::from_str(&script.state).unwrap_or(serde_json::json!({}));

            let mut ctx = ScriptContext {
                world: &self.registry.world,
                registry: self.registry.clone(),
            };

            match handler
                .run(&mut ctx, script.object_uid, parsed_state)
                .await
            {
                Ok(new_state) => {
                    self.registry
                        .world
                        .with_db(|db| db.scripts.record_run(&mut db.handle, script.id, &new_state))
                        .await
                        .ok();
                }
                Err(reason) => {
                    log::warn!(
                        "script {} (handler {:?}) returned err: {reason}; disabling",
                        script.id,
                        script.handler_key
                    );
                    self.registry
                        .world
                        .with_db(|db| db.scripts.disable(&mut db.handle, script.id))
                        .await
                        .ok();
                }
            }
        }
        Ok(())
    }
}
```

- [ ] **Step 2: Add `pub mod scheduler;` to `muoxi/src/server.rs` and the lib re-export**

In `muoxi/src/lib.rs` (created in Plan 4 Task 12), add `mod scheduler;` and `pub use crate::scheduler;` inside the `pub mod server` block.

- [ ] **Step 3: Verify**

Run: `cargo check -p muoxi`
Expected: `Finished`.

- [ ] **Step 4: Commit**

```bash
git add muoxi/src/server/scheduler.rs muoxi/src/lib.rs muoxi/src/server.rs
git commit -m "feat(server): Scheduler task + ScriptHandler trait"
```

---

## Task 7: Built-in `HeartbeatHandler` (worked example)

**Files:**
- Create: `muoxi/src/server/scripts/mod.rs`
- Create: `muoxi/src/server/scripts/heartbeat.rs`

- [ ] **Step 1: Write `scripts/mod.rs`**

```rust
//! Built-in script handlers. Downstream MUDs register their own via
//! `Registry::register_script_handler`.

pub mod heartbeat;

use crate::registry::Registry;
use std::sync::Arc;

/// Register every built-in script handler with `registry`.
pub fn register_all(registry: &Registry) {
    registry.register_script_handler(Arc::new(heartbeat::HeartbeatHandler));
}
```

- [ ] **Step 2: Write `scripts/heartbeat.rs`**

```rust
//! `heartbeat` — emits a log line every tick. Demonstrates the handler shape.
//!
//! Persistent state field `count`: total ticks since creation. Useful for
//! verifying scheduler liveness in CI.

use crate::scheduler::{ScriptContext, ScriptHandler};
use async_trait::async_trait;
use db::utils::UID;

#[derive(Debug)]
pub struct HeartbeatHandler;

#[async_trait]
impl ScriptHandler for HeartbeatHandler {
    fn key(&self) -> &'static str {
        "heartbeat"
    }

    async fn run(
        &self,
        _ctx: &mut ScriptContext<'_>,
        _object_uid: Option<UID>,
        state: serde_json::Value,
    ) -> Result<serde_json::Value, &'static str> {
        let count = state
            .get("count")
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
            + 1;
        log::info!(target: "muoxi::heartbeat", "tick {}", count);
        Ok(serde_json::json!({ "count": count }))
    }
}
```

- [ ] **Step 3: Add `pub mod scripts;` to `muoxi/src/server.rs` (and lib re-export)**

- [ ] **Step 4: Verify**

Run: `cargo check -p muoxi`
Expected: `Finished`.

- [ ] **Step 5: Commit**

```bash
git add muoxi/src/server/scripts/ muoxi/src/server.rs muoxi/src/lib.rs
git commit -m "feat(server): heartbeat ScriptHandler as worked example"
```

---

## Task 8: Spawn the scheduler at server startup + register built-ins

**Files:**
- Modify: `muoxi/src/server.rs`

- [ ] **Step 1: Update `main()`**

After the registry construction (Plan 4 Task 10), add:

```rust
crate::scripts::register_all(&registry);

let scheduler = crate::scheduler::Scheduler::new(registry.clone());
tokio::spawn(scheduler.run());
```

The scheduler runs forever in the background; the spawn handle is dropped because we want it to live as long as `main()` does.

- [ ] **Step 2: Verify build + smoke-test**

```bash
cd /home/duys/.repos/MuOxi
cargo build --bin muoxi_server
RUST_LOG=info,muoxi::heartbeat=debug ./target/debug/muoxi_server &
SERVER_PID=$!
sleep 1
# create a heartbeat script via a small test harness or via redis/manually-inserted row
# (For v0.1 we don't have an admin command for `script create` yet — manually insert via sqlite)
sqlite3 data/world.db <<'EOF'
INSERT INTO scripts(id, handler_key, interval_ms, next_run_at, repeat, state, enabled)
VALUES (1, 'heartbeat', 500, 0, 1, '{}', 1);
EOF

sleep 3
kill $SERVER_PID 2>/dev/null
```

Expected: server stderr shows multiple `tick N` log lines from the heartbeat handler. (Adjust `RUST_LOG` to ensure the log target is visible.)

- [ ] **Step 3: Commit**

```bash
git add muoxi/src/server.rs
git commit -m "feat(server): spawn Scheduler at startup; register built-in handlers"
```

---

## Task 9: Update root + db/server AGENTS.md

**Files:**
- Modify: root `AGENTS.md`
- Modify: `db/AGENTS.md`
- Modify: `muoxi/src/server/AGENTS.md`

- [ ] **Step 1: Root AGENTS.md**

In `/home/duys/.repos/MuOxi/AGENTS.md`:

- WHERE TO LOOK: add `Persistent timed events / scripts` → `muoxi/src/server/scheduler.rs` and `muoxi/src/server/scripts/`.
- CODE MAP: add `Script`, `ScriptRepo`, `ScriptHandler`, `Scheduler`.

- [ ] **Step 2: db/AGENTS.md**

Add `Script` and `ScriptRepo` to STRUCTURE + CORE TYPES. Add the `scripts` table to DATABASE SCHEMA.

- [ ] **Step 3: muoxi/src/server/AGENTS.md**

Append:

```markdown
## SCHEDULER

Persistent scheduled jobs live in the `scripts` table. The framework spawns a
single background `Scheduler` task at startup that polls the DB every 50ms
for due jobs and dispatches them to handlers registered via
`Registry::register_script_handler`.

Built-in handlers: `heartbeat` (counts ticks; demo). Downstream MUDs add their
own.

Schedule a job at runtime:

```rust
let s = registry.world.with_db(|db| {
    db.scripts.create_repeating(
        &mut db.handle,
        Some(mob_uid),         // owning object (None for global)
        "ai_tick",             // handler_key
        2_000,                 // interval_ms
        &serde_json::json!({}) // initial state
    )
}).await?;
```

Scripts survive restarts: on boot, the scheduler picks up overdue jobs and
runs them once before settling into steady-state polling.
```

- [ ] **Step 4: Commit**

```bash
git add AGENTS.md db/AGENTS.md muoxi/src/server/AGENTS.md
git commit -m "docs(agents): document scheduler / scripts subsystem"
```

---

## Verification Summary

A successful run of this plan ends with:

- [ ] `scripts` table exists in the migration set; `cargo test -p db --features db-sqlite --test integration_scripts` runs 4 passing tests.
- [ ] `Scheduler` task spawns at startup and polls every 50ms.
- [ ] Manually-inserted `heartbeat` script row produces log lines while the server runs.
- [ ] Disabling or deleting an object cascades to its scripts.
- [ ] `Registry::register_script_handler` + `script_handler(key)` work.
- [ ] Documentation reflects the new subsystem.
