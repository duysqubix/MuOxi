# MUOXI KNOWLEDGE BASE

**Generated:** 2026-05-07T00:32:44Z
**Commit:** 6c4384e (working tree, post-modernization)
**Branch:** master

## OVERVIEW

MUD/MU* online-text-game engine in Rust (edition 2024). Cargo workspace, 4 member crates. Tokio 1.x + Diesel 2.x + Redis 0.27 + tokio-tungstenite. Toolchain pinned to stable via [`rust-toolchain.toml`](file:///home/duys/.repos/MuOxi/rust-toolchain.toml). Was alpha-stage and dormant ~2020; dependency stack has been modernized.

## STRUCTURE

```
.
├── muoxi/         # 4-binary app crate (engine, staging, watchdog, web)
├── db/            # shared library: Postgres + Redis + JSON layer
├── benchmarks/    # custom non-Criterion benchmark harness
├── tester/        # sandbox/playground binary - NOT a test suite
├── migrations/    # Diesel SQL migrations (accounts, characters tables)
├── config/        # runtime ini (muoxi.ini)
├── json/          # canonical game data (accounts.json, characters.json)
├── resources/     # text assets (welcome.txt)
├── bin/           # empty marker dir
├── .media/        # logo (cog.png)
├── Cargo.toml              # workspace root + [workspace.dependencies]
├── rust-toolchain.toml     # pins toolchain to stable
├── docker-compose.yml      # dev stack: app + postgres + redis
├── Dockerfile              # rust builder + release stages (NEEDS UPDATE: still rust:1.47)
├── Dockerfile.postgres     # postgres init image
├── dev-entrypoint.sh       # container entrypoint
├── init-muoxi-db.sql       # creates muoxi DB + user
├── diesel.toml             # Diesel CLI -> db/src/schema.rs
└── .travis.yml             # legacy CI (NO active workflows)
```

## WHERE TO LOOK

| Task | Location |
|------|----------|
| TCP proxy / connection lifecycle | [`muoxi/src/staging/`](file:///home/duys/.repos/MuOxi/muoxi/src/staging/) |
| Game logic (echo only today) | [`muoxi/src/engine/muoxi.rs`](file:///home/duys/.repos/MuOxi/muoxi/src/engine/muoxi.rs) |
| JSON → Postgres sync | [`muoxi/src/watchdog/watchdog.rs`](file:///home/duys/.repos/MuOxi/muoxi/src/watchdog/watchdog.rs) |
| WebSocket bridge | [`muoxi/src/webserver/webserver.rs`](file:///home/duys/.repos/MuOxi/muoxi/src/webserver/webserver.rs) |
| Database tables / Diesel ORM | [`db/src/structures.rs`](file:///home/duys/.repos/MuOxi/db/src/structures.rs), [`db/src/schema.rs`](file:///home/duys/.repos/MuOxi/db/src/schema.rs) |
| Redis cache wrapper + key naming | [`db/src/cache_structures/`](file:///home/duys/.repos/MuOxi/db/src/cache_structures/) |
| Workspace dependency versions | [`Cargo.toml`](file:///home/duys/.repos/MuOxi/Cargo.toml) `[workspace.dependencies]` |
| Add new persistent table | new module under `db/src/`, edit `db/src/lib.rs`, write migration in `migrations/<date>_*/`, run `diesel migration run` |
| Welcome screen text | `resources/welcome.txt` |
| Add new comm protocol | new module in `muoxi/src/`, register binary in `muoxi/Cargo.toml` |
| Run benchmark | `cargo run --bin muoxi_benchmarks` (needs `benchmarks/db_100_000.json` fixture, NOT in repo) |

## CODE MAP

| Symbol | Location | Role |
|--------|----------|------|
| `DatabaseHandler` | `db/src/lib.rs` | Postgres connection wrapper; defaults `postgres://muoxi:muoxi@localhost/muoxi`. Diesel 2.x ops require `&mut handle`. |
| `Cache` | `db/src/cache.rs` | Redis connection factory; reads `REDIS_SERVER`, default `redis://127.0.0.1` |
| `CacheSocket` | `db/src/cache_structures/socket.rs` | Per-client Redis-backed socket state |
| `Cachable` trait | `db/src/cache_structures/mod.rs` | Redis serialize via `Type:UID:field` keys |
| `Account`, `Character` | `db/src/structures.rs` | Diesel ORM records; `#[diesel(table_name = ...)]` attributes |
| `Server`, `Client`, `Comms` | `muoxi/src/staging/comms.rs` | Connection state shared via `Arc<Mutex<Server>>` |
| `ConnStates` | `muoxi/src/staging/states.rs` | Login state machine (only `AwaitingName` implemented) |
| `Command` trait + `cmdset![]` macro | `muoxi/src/staging/prelude.rs` | Per-state command dispatch |
| `gen_uid()` | `db/src/utils.rs` | i64 UID = 32-bit unix-timestamp `<<` 32 \| 32-bit random |

## CONVENTIONS

- **Edition 2024** in every member crate. MSRV 1.85.
- **Tokio 1.x** + **Diesel 2.x** + **Redis 0.27**. Toolchain pinned to stable via `rust-toolchain.toml`.
- Async traits via `tokio::io::AsyncReadExt`/`AsyncWriteExt` (NO `tokio::prelude`). Stream extensions via `tokio_stream::StreamExt` and `futures_util::SinkExt`.
- `serde_json` always uses the `preserve_order` feature.
- Logging: `pretty_env_logger` 0.5 + `log` 0.4 macros; `RUST_LOG` set inside `staging_proxy::main` (wrapped in `unsafe { env::set_var(...) }` per modern std API).
- DB UID type: `i64` (`db::utils::UID`). Schema enforces `BIGINT NOT NULL CHECK (uid > 0)`.
- Diesel query connection arg is now `&mut PgConnection` (Diesel 2.x). All `DatabaseHandlerExt` methods take `conn: &mut PgConnection`.
- The `db` library is consumed by every other crate via `path = "../db"`.
- Multi-binary pattern: `muoxi` ships 4 bins via `[[bin]]` with custom paths into nested `src/<bin>/` dirs - NOT the standard `src/bin/`.
- Dependency versions are centralized in workspace root `[workspace.dependencies]`; each member crate just reads `{ workspace = true }`.

## ANTI-PATTERNS (THIS PROJECT)

- **DO NOT add `cargo test` workflows.** No `#[test]`/`#[cfg(test)]` exists anywhere; running tests today is a no-op. The `tester/` crate is a manual sandbox, not a test suite.
- **DO NOT use `tester/src/main.rs` as a code reference.** It is a deliberately small Redis round-trip; the original commented-out experiments were removed during modernization.
- **DO NOT change `db/src/schema.rs` by hand.** Regenerate via `diesel migration run` (output target set in `diesel.toml`).
- **DO NOT directly write to Postgres from app code.** Architectural rule: app writes to `json/`, `muoxi_watchdog` syncs JSON → Postgres. From the app's perspective Postgres is read-only.
- **DO NOT switch off `rust-toolchain.toml`.** Local nightly may regress with bleeding edge crate features; pinning to stable is the contract.

## COMMANDS

```bash
cargo build                           # workspace build (REQUIRES libpq-dev installed: `sudo apt install libpq-dev`)
cargo check --workspace               # type-check only, no linker (works without libpq)
cargo clippy --workspace --no-deps    # lints; current state passes with style warnings only
cargo run --bin muoxi_engine          # 127.0.0.1:4567 echo TCP server
cargo run --bin muoxi_staging         # 127.0.0.1:8000 staging proxy (needs libpq)
cargo run --bin muoxi_watchdog        # JSON-file watcher (needs libpq + Postgres running)
cargo run --bin muoxi_web             # ws://127.0.0.1:8080 → tcp 127.0.0.1:8000 bridge
cargo run --bin muoxi_sandbox         # tester crate (needs Redis running)
cargo run --bin muoxi_benchmarks      # benchmark crate (needs fixture; see benchmarks/AGENTS.md)

diesel migration run                   # apply migrations under migrations/
sudo -u postgres psql -v ON_ERROR_STOP=1 -f init-muoxi-db.sql

docker-compose up server               # full stack (Dockerfile is OUTDATED - still pins rust:1.47)
```

No `cargo fmt`, `cargo clippy` enforcement, `cargo test`, or active CI - none configured. `.travis.yml` retains only a legacy badge.

## ENV VARS

| Var | Default | Reader |
|-----|---------|--------|
| `DATABASE_URL` | `postgres://muoxi:muoxi@localhost/muoxi` | `DatabaseHandler::connect` |
| `REDIS_SERVER` | `redis://127.0.0.1` | `Cache::new` |
| `GAME_ADDR` | `127.0.0.1:4567` | `staging_proxy::main`, `engine::main` |
| `PROXY_ADDR` | `127.0.0.1:8000` | `staging_proxy::main`, `webserver::main` (forwards WS → TCP at PROXY_ADDR) |
| `WEB_ADDR` | `127.0.0.1:8080` | `webserver::main` (WS bind addr) |
| `RUST_LOG` | `info,warn,error,test` | set inside `staging_proxy::main` |

## NOTES / GOTCHAS

- **`libpq-dev` system dependency is REQUIRED** to LINK any binary that pulls in the `db` crate via Diesel/Postgres (`muoxi_staging`, `muoxi_watchdog`, `muoxi_sandbox`, `muoxi_benchmarks`). `cargo check` does not need it; `cargo build` does.
- `muoxi_engine` and `muoxi_web` build and run WITHOUT libpq (they don't pull diesel into their final binary).
- `muoxi_web` is now a clean `tokio-tungstenite` bridge: per-client WS-to-TCP forwarding. The README still claims "Not working as intended" - that's stale.
- `muoxi_web` listens on `WEB_ADDR` (default `127.0.0.1:8080`) and forwards to `PROXY_ADDR` (default `127.0.0.1:8000`). README says 8001 - stale.
- `muoxi_web` no longer serves HTML on `/`; it is WS-only. Old `static/index.html` reference is gone.
- `Dockerfile` still pins `rust:1.47.0-slim` and has a CMD typo `cargo run --bin " muoxi_web"` - both NEED updating to match the modernized stack but were out of scope.
- `dev-entrypoint.sh` runs `cargo install` on EVERY container start - guarded only by `tmp/setup.lock`.
- `staging_proxy.rs` reads `resources/welcome.txt` with a CWD-relative path - run binaries from repo root.
- `watchdog.rs` resolves `json/accounts.json` and `json/characters.json` from `env::current_dir()` - run from repo root. Now uses `std::sync::OnceLock` instead of `lazy_static`.
- `benchmarks/db_100_000.json` fixture is referenced but NOT committed - benchmark crate won't run out-of-the-box.
- README mentions MongoDB in the watchdog description; the actual code uses Postgres via Diesel. Stale doc.
- Removed: `db/src/clients.rs` (dead code, never wired through `lib.rs`).
- Removed: `muoxi/src/staging/copyover.rs` (replaced by `tokio::io::copy_bidirectional`).
- `redis::Commands::set_multiple` is deprecated; use `mset` (already done in `cache_structures/socket.rs`).
- Project owner notes the project was dormant ("two kids later") - expect long PR review cycles.
