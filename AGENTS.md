# Repo code map

This file is the per-repo orientation. New contributors and AI agents
should read this to find their way around. For end-user / framework-user
docs, see [`docs/`](docs/).

Per-subsystem invariants live in the co-located `AGENTS.md` files —
[`db/AGENTS.md`](db/AGENTS.md), [`muoxi/AGENTS.md`](muoxi/AGENTS.md),
[`muoxi/src/server/AGENTS.md`](muoxi/src/server/AGENTS.md).

## What this is

A Rust MUD framework. Cargo workspace, three member crates
(`muoxi`, `db`, `examples/extension`). Tokio 1.x + Diesel 2.x (SQLite
default, Postgres opt-in) + Redis 0.27 + tokio-tungstenite. Toolchain
pinned to stable via [`rust-toolchain.toml`](rust-toolchain.toml). Default
build needs zero system packages thanks to bundled `libsqlite3-sys`.

For the design rationale see [`docs/architecture.md`](docs/architecture.md).

## Tree

```
.
├── muoxi/              # application crate (bins + lib)
├── db/                 # persistence library (Diesel + Redis)
├── examples/extension/ # downstream MUD embedding demo
├── migrations/         # Diesel SQL — embedded at compile time
├── resources/          # welcome banner + browser test client
├── data/               # SQLite world.db lands here at runtime (gitignored)
├── docs/               # end-user / framework-user documentation
├── .media/             # logo asset
├── Cargo.toml          # workspace root + [workspace.dependencies]
├── Cargo.lock          # tracked (workspace ships binaries)
├── rust-toolchain.toml # pinned stable
├── docker-compose.yml  # dev stack: muoxi_server + muoxi_web + redis
├── Dockerfile          # multi-stage rust:1.85 builder + slim runtime
└── diesel.toml         # Diesel CLI → db/src/schema.rs
```

## Where to look

| Task | Location |
| --- | --- |
| TCP server / connection lifecycle / state machine | [`muoxi/src/server/`](muoxi/src/server/) |
| Game-logic dispatch (Registry → lock check → command) | [`muoxi/src/server/cmds.rs`](muoxi/src/server/cmds.rs) |
| WebSocket bridge + browser test client | [`muoxi/src/webserver/webserver.rs`](muoxi/src/webserver/webserver.rs) |
| Account table + auth helpers | [`db/src/structures.rs`](db/src/structures.rs), [`muoxi/src/server/auth.rs`](muoxi/src/server/auth.rs) |
| Generic in-world objects (rooms / items / mobs / etc.) | [`db/src/objects/`](db/src/objects/) |
| Backend selection (sqlite vs postgres feature) | [`db/src/conn.rs`](db/src/conn.rs) |
| Redis cache wrapper + key naming | [`db/src/cache_structures/`](db/src/cache_structures/) |
| Workspace dependency versions | [`Cargo.toml`](Cargo.toml) `[workspace.dependencies]` |
| Welcome screen text | [`resources/welcome.txt`](resources/welcome.txt) |
| Adding a new persistent table | new module under `db/src/`, edit `db/src/lib.rs`, write migration in `migrations/<date>_*/` |
| Adding a new comm protocol | new module in `muoxi/src/`, register binary in `muoxi/Cargo.toml` |

## Code map

| Symbol | Location | Role |
| --- | --- | --- |
| `Conn` | [`db/src/conn.rs`](db/src/conn.rs) | Type alias resolved by active feature: `SqliteConnection` (default) or `PgConnection`. Reads `DATABASE_URL`; defaults to `data/world.db`. |
| `DatabaseHandler` | [`db/src/lib.rs`](db/src/lib.rs) | Holds one `Conn` + `accounts`, `objects`, `attributes`, `tags`, `character_accounts` handlers. Runs embedded migrations on `connect()`. |
| `Cache` | [`db/src/cache.rs`](db/src/cache.rs) | Redis connection factory; reads `REDIS_SERVER`. |
| `CacheSocket` | [`db/src/cache_structures/socket.rs`](db/src/cache_structures/socket.rs) | Per-client Redis-backed socket state. |
| `Cachable` trait | [`db/src/cache_structures/mod.rs`](db/src/cache_structures/mod.rs) | Redis (de)serialize via `Type:UID:field` keys. |
| `Account` | [`db/src/structures.rs`](db/src/structures.rs) | Login identity Diesel record. |
| `Object`, `ObjectRepo` | [`db/src/objects/object.rs`](db/src/objects/object.rs) | Generic in-world entity (`type_key` discriminates). |
| `ObjectAttribute`, `AttributeRepo` | [`db/src/objects/attribute.rs`](db/src/objects/attribute.rs) | Per-object key → JSON-text bag. |
| `ObjectTag`, `TagRepo` | [`db/src/objects/tag.rs`](db/src/objects/tag.rs) | Per-object `(key, category)` labels. |
| `CharacterAccount`, `CharacterAccountRepo` | [`db/src/objects/character_account.rs`](db/src/objects/character_account.rs) | Character ⇄ Account link with ordinal. |
| `Server`, `Client`, `Comms` | [`muoxi/src/server/comms.rs`](muoxi/src/server/comms.rs) | Per-connection state. `Server.clients` is the shared roster. |
| `ConnStates` | [`muoxi/src/server/states.rs`](muoxi/src/server/states.rs) | 8-state login + character-select machine. |
| `auth::{hash_password, verify_password, AuthBuffer}` | [`muoxi/src/server/auth.rs`](muoxi/src/server/auth.rs) | argon2id + per-session credential scratch + validators. |
| `Registry` | [`muoxi/src/server/registry.rs`](muoxi/src/server/registry.rs) | Central index of TypeClasses, Commands, Hooks. |
| `WorldApi` | [`muoxi/src/server/world.rs`](muoxi/src/server/world.rs) | DB facade for command/hook handlers. |
| `TypeClass` + 5 built-ins | [`muoxi/src/server/typeclass.rs`](muoxi/src/server/typeclass.rs) | In-world type defs (character / room / item / exit / mob). |
| `Hook` + `Hooks` | [`muoxi/src/server/hooks.rs`](muoxi/src/server/hooks.rs) | Lifecycle event listeners. Only `at_login`/`at_disconnect` are fired today. |
| `Command` + `CommandContext` | [`muoxi/src/server/prelude.rs`](muoxi/src/server/prelude.rs) | Player command trait + per-invocation context. |
| Built-in commands | [`muoxi/src/server/commands/`](muoxi/src/server/commands/) | `look`, `say`, `quit`, `who`. |
| `gen_uid()` | [`db/src/utils.rs`](db/src/utils.rs) | `i64` UID = `(unix_secs << 32) | rand32`. |

## Conventions

- **Edition 2024**, stable Rust 1.85, pinned via `rust-toolchain.toml`.
- **Tokio 1.x** — individual `AsyncReadExt` / `AsyncWriteExt` imports, no `tokio::prelude`.
- **Diesel 2.x** — every query takes `&mut Conn`, macros namespaced (`diesel::table!`).
- **SQLite default** — Postgres opt-in via `--features db-postgres` (requires `libpq-dev`). A compile-error guard in `db/src/conn.rs` enforces exactly one backend.
- **`db` crate is `#![deny(missing_docs)]`** — every public item gets a docstring.
- **Workspace deps centralized** in `Cargo.toml` `[workspace.dependencies]`; members reference via `{ workspace = true }`.
- **Commands and hooks go through `WorldApi`** — never `diesel::insert_into(...)` directly. Add typed methods to `WorldApi` if needed.

## Things that catch people out

- `db/src/schema.rs` is generated; regenerate via `diesel migration run`
  rather than editing it by hand. The output target is set in
  `diesel.toml`.
- The DB is the single source of truth. JSON files under `json/` are for
  import/export payloads only.
- There is no separate `characters` table. Characters are `objects` with
  `type_key = "character"`; the account link lives in `character_accounts`.
- The core schema stays portable between SQLite and Postgres — avoid
  Postgres-only types (`BIGINT[]`, `JSONB`, `LISTEN/NOTIFY`, `to_tsvector`).
- `redis::Commands::set_multiple` is deprecated in 0.27; use `mset`.
- `DatabaseHandler::connect()` is Diesel-2.x sync-blocking. From async
  contexts, offload it.

## Commands

```bash
cargo build --workspace                                              # SQLite default, no system deps
cargo build --no-default-features --features db-postgres             # Postgres (needs libpq-dev)
cargo check --workspace
cargo clippy --workspace --no-deps
cargo test -p db --features db-sqlite                                # in-memory SQLite roundtrip tests
cargo test -p muoxi --test registry                                  # registry smoke tests
cargo test -p muoxi --lib auth                                       # argon2 + validators

cargo run --bin muoxi_server                                         # 127.0.0.1:8000 unified server
cargo run --bin muoxi_web                                            # ws://127.0.0.1:8080 + browser test client
cargo run --bin muoxi-example-extension                              # downstream-MUD embedding demo

docker compose up                                                    # full dev stack
DEV_AUTOLOGIN=1 docker compose up                                    # skip auth for fast iteration
```

## Env vars

| Var | Default | Reader |
| --- | --- | --- |
| `DATABASE_URL` | `data/world.db` (SQLite) / `postgres://muoxi:muoxi@localhost/muoxi` (PG) | `db::conn::establish` |
| `REDIS_SERVER` | `redis://127.0.0.1` | `Cache::new` |
| `PROXY_ADDR` | `127.0.0.1:8000` | `muoxi_server`, `muoxi_web` (outbound target) |
| `WEB_ADDR` | `127.0.0.1:8080` | `muoxi_web` (bind) |
| `DEV_AUTOLOGIN` | unset | `muoxi_server` — when set, skips auth state machine |
| `RUST_LOG` | `info,warn,error,test` | forced inside `muoxi_server::main` |

## Notes

- SQLite WAL mode + foreign keys are enabled automatically (see `db::conn::configure`).
- Embedded migrations run on `DatabaseHandler::connect()` — no `diesel migration run` needed at runtime.
- Schema is portable across SQLite and Postgres backends.
- `muoxi_web` is dual-purpose on a single port: HTTP GET returns the browser test client (`resources/web/index.html`, embedded at compile time); WS upgrade bridges to TCP backend.
- `muoxi_server` binds to `127.0.0.1` by default — set `PROXY_ADDR=0.0.0.0:8000` (or use the docker compose path) for external access.
