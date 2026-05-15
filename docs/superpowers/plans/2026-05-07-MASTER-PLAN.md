# MuOxi v0.1 Master Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement each axis-plan task-by-task. This master document is the index — execute the linked axis-plans in dependency order.

---

## ⚠️ RESUME POINT (read this first if you're picking up a fresh session)

**As of commit `e40d829` on master:**

| Plan | Status | Commits |
|---|---|---|
| 1. SQLite migration + scrap JSON/watchdog | ✅ **COMPLETE** | `7251594`..`fefae04` (14 commits) |
| 2. Topology collapse → `muoxi_server` | ✅ **COMPLETE** | `77bc3d8`..`5972110` (3 commits) |
| 3. Object/Attribute/Tag model | ✅ **COMPLETE** | `e4d089c`..`53e4165` (11 commits) |
| 4. Command/Hook registry | ⏳ Not started | — |
| 5. Persistent scheduler | ⏳ Not started | — |
| 6. Complete auth state machine | ⏳ Not started | — |

**To resume:**
1. `git pull origin master` — get to `e40d829` or later
2. `cargo build --workspace` should succeed without `libpq-dev` (sanity check)
3. `cargo test -p db --features db-sqlite` — should show `account_roundtrip ... ok`
4. Open the next axis-plan in dependency order — **start with Plan 3** (`2026-05-07-object-attribute-model.md`)
5. Execute via `superpowers:subagent-driven-development` or inline per `superpowers:executing-plans`

**Things that drifted between original plan and reality (worth knowing):**
- Plan 2 originally wrote the binary entrypoint at `muoxi/src/server.rs`. Rust's module resolution for non-standard binary paths requires sub-modules to be **siblings** of the entry file. Solution applied during execution: move to `muoxi/src/server/main.rs` so that `pub mod cmds;` etc. resolve correctly. **Plans 4–6 reference `server/main.rs` accordingly.**
- `account_characters` was created by Plan 1's migration. Plan 3's migration drops it and replaces with `character_accounts` (linking objects, not characters). This is the expected schema evolution — already encoded in Plan 3.
- The `db` crate now re-exports `diesel` via `pub use diesel;` (added during Plan 1 Task 4). Downstream crates can use `use db::diesel::prelude::*;` without a direct `diesel` dep.

---

**Goal:** Transform MuOxi from "modernized Rust workspace with login scaffolding" into a credible v0.1 of an Evennia-class MUD framework in Rust.

**Architecture:** Replace the JSON-canonical / watchdog-mirror persistence model with SQLite-as-source-of-truth (Postgres optional via Cargo feature). Collapse the staging/engine binary split into a single server. Introduce a generic Object + Attribute + Tag domain model so downstream developers can extend the world model without schema migrations. Add a command/hook registry as the extension surface and a persistent scheduler for timed events. Defer scripting (PyO3/mlua) to v0.2.

**Tech Stack:** Rust edition 2024, Tokio 1.x, Diesel 2.x with `sqlite` (default) and `postgres` (optional) backends, Redis 0.27 (transient cache, unchanged), tokio-tungstenite 0.24 (websocket bridge, unchanged), bundled `libsqlite3-sys` for zero-system-dep builds.

---

## Decisions Locked In

| Axis | Choice | Why |
|---|---|---|
| Persistence default | SQLite via Diesel `sqlite` feature, with `libsqlite3-sys` `bundled` | Zero-config, no `libpq-dev`, framework-adoption posture matches Evennia |
| Persistence opt-in | Postgres via Cargo feature `db-postgres` | Production deployments; framework still supports the Postgres path |
| JSON canonical | Removed | File-level writes are not transactional; whole-file rewrite doesn't scale; single point of failure via `hotwatch` |
| Watchdog binary | Deleted | Has no purpose once Postgres stops being a mirror |
| `Account.characters BIGINT[]` | Replaced by `account_characters(account_uid, character_uid, ordinal)` join table | Postgres-only type; bad relational shape on either backend |
| v0.1 process topology | Single `muoxi_server` binary (staging + engine logic merged) | No hot-reload story exists yet, so the split has no benefit; can re-split in v0.2 |
| Object model | Generic `objects` + `object_attributes` + `object_tags` tables | Evennia-style; downstream games extend without schema migrations |
| Account/Character | Specialized objects with FK to `objects.uid` | Stable typed core for auth, freeform attributes for everything else |
| Hot-reload | Out of scope for v0.1 | Hard in Rust; defer until scripting layer exists |
| Scripting language | Out of scope for v0.1 | Stabilize Rust `WorldApi` first; PyO3 vs mlua is v0.2 |
| Existing auth state machine | Completed (all 8 `ConnStates` implemented) | Was 1/8 working; auth flow blocks everything downstream |

## Axis Plans

Execute in this order — each plan unblocks the next:

| # | Plan | File | Depends On |
|---|---|---|---|
| 1 | SQLite migration + JSON/watchdog removal | [`2026-05-07-sqlite-migration.md`](./2026-05-07-sqlite-migration.md) | — |
| 2 | Collapse staging+engine into `muoxi_server` | [`2026-05-07-collapse-topology.md`](./2026-05-07-collapse-topology.md) | — (parallel-safe with 1) |
| 3 | Generic Object/Attribute/Tag model | [`2026-05-07-object-attribute-model.md`](./2026-05-07-object-attribute-model.md) | 1 |
| 4 | Command/Hook registry (extension surface) | [`2026-05-07-command-hook-registry.md`](./2026-05-07-command-hook-registry.md) | 3 |
| 5 | Persistent scheduler / scripts subsystem | [`2026-05-07-scheduler.md`](./2026-05-07-scheduler.md) | 3 |
| 6 | Complete the auth state machine | [`2026-05-07-auth-state-machine.md`](./2026-05-07-auth-state-machine.md) | 3, 4 |

Plans 1 and 2 can run in either order or in parallel (different files). Plans 3–6 are strictly sequential.

## Recommended Build Order

1. **SQLite migration** (`2026-05-07-sqlite-migration.md`) — foundational, deletes the most code, biggest reduction in friction.
2. **Topology collapse** (`2026-05-07-collapse-topology.md`) — smaller binary surface for everything that follows.
3. **Object/Attribute/Tag model** (`2026-05-07-object-attribute-model.md`) — defines the framework's domain.
4. **Command/Hook registry** (`2026-05-07-command-hook-registry.md`) — defines the extension surface.
5. **Scheduler** (`2026-05-07-scheduler.md`) — adds timed/persistent behavior.
6. **Auth state machine** (`2026-05-07-auth-state-machine.md`) — uses everything above.

## Definition of Done for v0.1

- [ ] `cargo build --workspace` succeeds on a fresh checkout with **no system packages installed** (no libpq, no postgres). Default `db-sqlite` feature.
- [ ] `cargo build --workspace --no-default-features --features db-postgres` still succeeds when libpq is available.
- [ ] `cargo run --bin muoxi_server` starts, binds the configured ports, and accepts a telnet client through the full login flow (new account creation, password set, character creation, character select).
- [ ] `cargo run --bin muoxi_web` bridges WebSocket clients into the same login flow end-to-end.
- [ ] Downstream developer can create a new command by impl'ing `Command` and registering it via `Registry::register_command`. Worked example included in `examples/`.
- [ ] Downstream developer can create a new in-game object type by registering a `TypeClass` with default attributes, hooks, and command set. Worked example included.
- [ ] At least one example timed script (e.g., heartbeat tick, weather rotation) ships and runs.
- [ ] All existing `cargo clippy` warnings resolved or explicitly `#[allow(...)]` with comment.
- [ ] AGENTS.md files updated to reflect the new architecture.
- [ ] README.md updated. Installation requires only `cargo` — no apt/brew/postgres prerequisites.

## Out of Scope for v0.1

These are recognized goals but explicitly deferred:

- Hot-reload of game code (portal/server protocol comes back in v0.2).
- Embedded scripting (PyO3 / mlua / rhai — v0.2).
- MCCP telnet compression (v0.2).
- Chat channels persistence (v0.2 — basic in-memory channels OK in v0.1).
- World import/export tooling (v0.2).
- Web admin UI (v0.3+).
- Spatial/zone/area abstractions beyond simple `location` references (v0.3+).
- Combat system, skills, classes (downstream-game concern, not framework).

## Sequencing Notes for Executors

- Each axis-plan is self-contained: file paths, code, tests, commits.
- After completing each plan, run `cargo check --workspace && cargo clippy --workspace --no-deps && cargo build --workspace` before starting the next plan.
- Each plan ends with a "Verification" section listing the commands to run and expected output.
- Use `superpowers:subagent-driven-development` to dispatch a fresh subagent per task, with two-stage review between tasks. The plans are bite-sized enough to support this.
- Or use `superpowers:executing-plans` for inline batch execution if you prefer a single session.

## Cross-Plan Fixups (one-liners worth knowing during execution)

These are small but real — apply opportunistically, not as separate tasks:

- **`pub use diesel;` in `db/src/lib.rs`** (during Plan 1 Task 4): so `WorldApi::with_db` callers in Plans 5/6 can write `use db::diesel::prelude::*;` instead of forcing the `muoxi` crate to add a direct `diesel` dependency. Single line.
- **Plan 6 Task 6's password-hash lookup** can be simplified once `find_account_by_name` exists (Plan 6 Task 5): replace the inline `with_db(|db| ... select(password_hash) ...)` with `world.find_account_by_name(name).await.map(|a| a.password_hash)`. Cleaner; no behavioral change.
- **Plan 4 Task 4's `Hooks::emit` closure ergonomics** are awkward when fired from inside async fns that capture `&mut Client`. If the awkwardness becomes painful in Plan 6, change `Hooks::emit` to take a single `Arc<dyn Hook>` per call (caller iterates) rather than the FnMut Future-builder shape — about 10 lines of API delta.
- **`tester` and `benchmarks` crates** are not exercised by these plans. After the dust settles, consider deleting `tester` (its purpose is replaced by the integration tests added across plans 1-6) and the bench harness can be replaced by a 10-line `criterion` setup if needed.

## Risk Register

| Risk | Mitigation |
|---|---|
| Diesel 2.x SQLite has subtle differences from Postgres (no `RETURNING` by default, no arrays) | Plan 1 introduces only portable schema. Plan 3's object model uses `TEXT` JSON values, not `JSONB`. |
| Existing `ConnStates::execute` falls through to `Quit` for 7/8 states | Plan 6 implements all 8. No partial state machine ever ships. |
| Generic object model could over-abstract | Each TypeClass starts as a thin wrapper around `Object` with default attributes/hooks. No deep inheritance hierarchy. |
| Command/hook merging could become hairy | Plan 4 defines exact merge semantics (priority, replace, parent) before any code. |
| Scheduler reliability across restarts | Plan 5 stores `next_run_at` in DB and runs a recovery scan on boot. |
| Topology collapse loses the v0.2 portal/server upgrade path | Plan 2 keeps the framing-protocol design notes in code comments and AGENTS.md so the v0.2 reintroduction is straightforward. |
