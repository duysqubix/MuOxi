# MUOXI KNOWLEDGE BASE

**Generated:** 2026-05-07T00:32:44Z
**Commit:** 6c4384e (working tree, post-modernization)
**Branch:** master

## OVERVIEW

MUD/MU* online-text-game engine in Rust (edition 2024). Cargo workspace, 4 member crates. Tokio 1.x + Diesel 2.x (**SQLite default**, Postgres optional via `db-postgres` feature) + Redis 0.27 + tokio-tungstenite. Toolchain pinned to stable via [`rust-toolchain.toml`](file:///home/duys/.repos/MuOxi/rust-toolchain.toml). Default builds need **zero system packages** thanks to bundled `libsqlite3-sys`.

## STRUCTURE

```
.
├── muoxi/         # 2-binary app crate (server, web)
├── db/            # shared library: SQLite/Postgres + Redis
├── benchmarks/    # custom non-Criterion benchmark harness
├── tester/        # sandbox/playground binary - NOT a test suite
├── migrations/    # Diesel SQL migrations (accounts + objects/attributes/tags/character_accounts)
├── config/        # runtime ini (muoxi.ini)
├── data/          # SQLite world.db lives here at runtime (gitignored)
├── json/          # import/export payloads only (NOT runtime canonical state)
├── resources/     # text assets (welcome.txt)
├── bin/           # empty marker dir
├── .media/        # logo (cog.png)
├── Cargo.toml              # workspace root + [workspace.dependencies]
├── rust-toolchain.toml     # pins toolchain to stable
├── docker-compose.yml      # dev stack: server + redis (no postgres)
├── Dockerfile              # multi-stage rust:1.85 builder + slim runtime
├── dev-entrypoint.sh       # container entrypoint (exec passthrough)
├── diesel.toml             # Diesel CLI -> db/src/schema.rs
└── .travis.yml             # legacy CI (NO active workflows)
```

## WHERE TO LOOK

| Task | Location |
|------|----------|
| TCP proxy / connection lifecycle | [`muoxi/src/server/`](file:///home/duys/.repos/MuOxi/muoxi/src/server/) |
| Game logic (echo only today) | [`muoxi/src/server/engine.rs`](file:///home/duys/.repos/MuOxi/muoxi/src/server/engine.rs) |
| WebSocket bridge | [`muoxi/src/webserver/webserver.rs`](file:///home/duys/.repos/MuOxi/muoxi/src/webserver/webserver.rs) |
| Account table / Diesel ORM | [`db/src/structures.rs`](file:///home/duys/.repos/MuOxi/db/src/structures.rs), [`db/src/schema.rs`](file:///home/duys/.repos/MuOxi/db/src/schema.rs) |
| Generic in-world objects (rooms/items/mobs/characters) | [`db/src/objects/`](file:///home/duys/.repos/MuOxi/db/src/objects/) |
| Backend selection + connection | [`db/src/conn.rs`](file:///home/duys/.repos/MuOxi/db/src/conn.rs) |
| Redis cache wrapper + key naming | [`db/src/cache_structures/`](file:///home/duys/.repos/MuOxi/db/src/cache_structures/) |
| Workspace dependency versions | [`Cargo.toml`](file:///home/duys/.repos/MuOxi/Cargo.toml) `[workspace.dependencies]` |
| Add new persistent table | new module under `db/src/`, edit `db/src/lib.rs`, write migration in `migrations/<date>_*/`, run `diesel migration run` |
| Welcome screen text | `resources/welcome.txt` |
| Add new comm protocol | new module in `muoxi/src/`, register binary in `muoxi/Cargo.toml` |
| Run benchmark | `cargo run --bin muoxi_benchmarks` (needs `benchmarks/db_100_000.json` fixture, NOT in repo) |

## CODE MAP

| Symbol | Location | Role |
|--------|----------|------|
| `Conn` (type alias) | `db/src/conn.rs` | `SqliteConnection` (default) or `PgConnection` (with `db-postgres`). Reads `DATABASE_URL`; defaults to `data/world.db` for SQLite. |
| `DatabaseHandler` | `db/src/lib.rs` | Holds one `Conn` + `accounts`, `objects`, `attributes`, `tags`, `character_accounts` handlers. Diesel 2.x ops require `&mut handle`. |
| `Cache` | `db/src/cache.rs` | Redis connection factory; reads `REDIS_SERVER`, default `redis://127.0.0.1` |
| `CacheSocket` | `db/src/cache_structures/socket.rs` | Per-client Redis-backed socket state |
| `Cachable` trait | `db/src/cache_structures/mod.rs` | Redis serialize via `Type:UID:field` keys |
| `Account` | `db/src/structures.rs` | Login identity Diesel record |
| `Object`, `ObjectRepo` | `db/src/objects/object.rs` | Generic in-world entity (`type_key` discriminates rooms/items/characters/...). Repo wraps Diesel CRUD. |
| `ObjectAttribute`, `AttributeRepo` | `db/src/objects/attribute.rs` | Per-object key→JSON-text bag. Values are `serde_json::Value` at the Rust API boundary. |
| `ObjectTag`, `TagRepo` | `db/src/objects/tag.rs` | Per-object (key, category) labels; idempotent add, cross-object lookup. |
| `CharacterAccount`, `CharacterAccountRepo` | `db/src/objects/character_account.rs` | Link table: character object → owning account, with ordinal. |
| `Server`, `Client`, `Comms` | `muoxi/src/server/comms.rs` | Connection state shared via `Arc<Mutex<Server>>` |
| `ConnStates` | `muoxi/src/server/states.rs` | Login state machine (only `AwaitingName` + `Playing` implemented; Plan 6 finishes the rest) |
| `Registry` | `muoxi/src/server/registry.rs` | Central index of TypeClasses, Commands, Hooks. Threaded into every session via `Arc<Registry>`. |
| `WorldApi` | `muoxi/src/server/world.rs` | DB facade for command handlers. Wraps `DatabaseHandler` in `Arc<Mutex<>>`. |
| `TypeClass` trait + 5 built-ins | `muoxi/src/server/typeclass.rs` | In-world type definitions (Character, Room, Item, Exit, Mob). |
| `Hook` trait + `Hooks` | `muoxi/src/server/hooks.rs` | Lifecycle event listeners (at_login, at_disconnect, at_pre/post_move, ...) |
| `Command` trait + `CommandContext` | `muoxi/src/server/prelude.rs` | Per-state command dispatch via the Registry |
| Built-in commands | `muoxi/src/server/commands/` | `look`, `say`, `quit`, `who` |
| `gen_uid()` | `db/src/utils.rs` | i64 UID = 32-bit unix-timestamp `<<` 32 \| 32-bit random |

## CONVENTIONS

- **Edition 2024** in every member crate. MSRV 1.85.
- **Tokio 1.x** + **Diesel 2.x** + **Redis 0.27**. Toolchain pinned to stable via `rust-toolchain.toml`.
- Async traits via `tokio::io::AsyncReadExt`/`AsyncWriteExt` (NO `tokio::prelude`). Stream extensions via `tokio_stream::StreamExt` and `futures_util::SinkExt`.
- `serde_json` always uses the `preserve_order` feature.
- Logging: `pretty_env_logger` 0.5 + `log` 0.4 macros; `RUST_LOG` set inside `staging_proxy::main` (wrapped in `unsafe { env::set_var(...) }` per modern std API).
- DB UID type: `i64` (`db::utils::UID`). Schema enforces `BIGINT NOT NULL CHECK (uid > 0)`.
- Diesel query connection arg is now `&mut Conn` (Diesel 2.x). All `DatabaseHandlerExt` methods take `conn: &mut Conn`.
- The `db` library is consumed by every other crate via `path = "../db"`.
- Multi-binary pattern: `muoxi` ships 2 bins (`muoxi_server`, `muoxi_web`) via `[[bin]]` with custom paths into nested `src/<bin>/` dirs - NOT the standard `src/bin/`.
- Dependency versions are centralized in workspace root `[workspace.dependencies]`; each member crate just reads `{ workspace = true }`.
- **Backend is SQLite by default**. Postgres requires `--no-default-features --features db-postgres` AND `libpq-dev` on the host.

## ANTI-PATTERNS (THIS PROJECT)

- **DO NOT add `cargo test` workflows.** Integration tests live under `db/tests/`; ordinary unit `#[test]` modules are sparse. Run with `cargo test -p db --features db-sqlite`.
- **DO NOT use `tester/src/main.rs` as a code reference.** It is a deliberately small Redis round-trip.
- **DO NOT change `db/src/schema.rs` by hand.** Regenerate via `diesel migration run` (output target set in `diesel.toml`).
- **DO NOT bring back JSON-canonical / watchdog.** Database is the single source of truth. `json/` is import/export only.
- **DO NOT add a separate `characters` table.** Characters are objects with `type_key = "character"`; the account link lives in `character_accounts`.
- **DO NOT bypass the repos.** Engine code calls `db.objects.create(...)`, never `diesel::insert_into(objects::table)` directly.
- **DO NOT switch off `rust-toolchain.toml`.** Local nightly may regress with bleeding edge crate features; pinning to stable is the contract.

## COMMANDS

```bash
cargo build --workspace               # SQLite default; zero system packages needed
cargo build --workspace --no-default-features --features db-postgres  # requires libpq-dev
cargo check --workspace               # link-free type check
cargo clippy --workspace --no-deps    # lints; style warnings only
cargo test -p db --features db-sqlite # SQLite in-memory integration tests

cargo run --bin muoxi_server          # 127.0.0.1:8000 unified server (login + game)
cargo run --bin muoxi_web             # ws://127.0.0.1:8080 → tcp 127.0.0.1:8000 bridge
cargo run --bin muoxi_sandbox         # tester crate (needs Redis running)
cargo run --bin muoxi_benchmarks      # benchmark crate (needs fixture; see benchmarks/AGENTS.md)

docker compose up server              # SQLite-backed dev stack (server + redis)
```

The first run creates `data/world.db` (SQLite). With `db-postgres`, set `DATABASE_URL` and run `diesel migration run` manually (CLI install: `cargo install diesel_cli --no-default-features --features postgres`).

## ENV VARS

| Var | Default | Reader |
|-----|---------|--------|
| `DATABASE_URL` | `data/world.db` (SQLite) / `postgres://muoxi:muoxi@localhost/muoxi` (PG) | `db::conn::establish` |
| `REDIS_SERVER` | `redis://127.0.0.1` | `Cache::new` |
| `PROXY_ADDR` | `127.0.0.1:8000` | `server::main`, `webserver::main` (forwards WS → TCP at PROXY_ADDR) |
| `WEB_ADDR` | `127.0.0.1:8080` | `webserver::main` (WS bind addr) |
| `RUST_LOG` | `info,warn,error,test` | set inside `server::main` |

## NOTES / GOTCHAS

- **Default builds need zero system packages.** SQLite ships bundled via `libsqlite3-sys` with the `bundled` feature. Postgres backend requires `libpq-dev` ONLY when selecting `--features db-postgres`.
- `muoxi_web` listens on `WEB_ADDR` (default `127.0.0.1:8080`) and forwards to `PROXY_ADDR` (default `127.0.0.1:8000`).
- `muoxi_web` is WS-only (no HTML serving).
- `server/main.rs` reads `resources/welcome.txt` with a CWD-relative path - run binaries from repo root.
- `benchmarks/db_100_000.json` fixture is referenced but NOT committed - benchmark crate won't run out-of-the-box.
- Removed: `db/src/clients.rs`, `muoxi/src/staging/copyover.rs`, `muoxi/src/watchdog/`, `muoxi/src/engine/`, `muoxi/src/staging/` (merged into `muoxi/src/server/`), `Dockerfile.postgres`, `init-muoxi-db.sql`, `.postgres-setup`.
- SQLite WAL mode + foreign keys are enabled automatically at connection time (see `db::conn::configure`).
- Account.characters BIGINT[] (PG-only) was replaced first by `account_characters` (Plan 1) and then by `character_accounts` (Plan 3, linking object UIDs since characters are now Objects).
- Schema migrations apply identically to SQLite and Postgres backends.
