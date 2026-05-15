# Development

For hacking on MuOxi itself. If you're using MuOxi to build a MUD, read
[getting-started.md](getting-started.md) and
[extension-guide.md](extension-guide.md) instead.

## Setup

```bash
git clone https://github.com/duysqubix/MuOxi.git
cd MuOxi
cargo build --workspace
```

`rustup` fetches the toolchain pinned in
[`rust-toolchain.toml`](../rust-toolchain.toml). No system packages
needed for the default SQLite build. `libpq-dev` is required only if
you intentionally switch to the Postgres feature.

## Sanity check

```bash
cargo check --workspace
cargo test -p db --features db-sqlite
cargo test -p muoxi --test registry
cargo test -p muoxi --lib auth
cargo clippy --workspace --no-deps
```

If those all pass, you have a clean baseline.

## The two crates

`db/` is the persistence library — Diesel and Redis, feature-gated
between SQLite (default) and Postgres. No server code here.

`muoxi/` is the application crate, with two binaries
(`muoxi_server` and `muoxi_web`) plus a library half that exports the
extension surface for downstream consumers and tests.

The lib half lives in [`muoxi/src/lib.rs`](../muoxi/src/lib.rs) and
uses `#[path]` attributes to pull modules from `muoxi/src/server/`.
That lets the binary at `muoxi/src/server/main.rs` and external crates
both reach the same module tree.

## Local invariants

Each subsystem has an `AGENTS.md` documenting the conventions and the
things that have caught people out. Worth a glance before touching the
subsystem.

- [Root `AGENTS.md`](../AGENTS.md) — repo-wide code map
- [`db/AGENTS.md`](../db/AGENTS.md) — Diesel patterns, schema rules,
  backend feature gating, Redis conventions
- [`muoxi/AGENTS.md`](../muoxi/AGENTS.md) — binary layout, lib+bin
  split, Tokio import gotchas
- [`muoxi/src/server/AGENTS.md`](../muoxi/src/server/AGENTS.md) —
  state machine, command dispatch, hook firing, lock evaluator

## Where things live

| You want to change | Look in |
| --- | --- |
| The connection-state machine | [`muoxi/src/server/states.rs`](../muoxi/src/server/states.rs) |
| Command dispatch (resolve → lock → execute) | [`muoxi/src/server/cmds.rs`](../muoxi/src/server/cmds.rs) |
| The `Command` / `CommandContext` trait | [`muoxi/src/server/prelude.rs`](../muoxi/src/server/prelude.rs) |
| The `Hook` trait or firing semantics | [`muoxi/src/server/hooks.rs`](../muoxi/src/server/hooks.rs) |
| The `TypeClass` trait or built-in types | [`muoxi/src/server/typeclass.rs`](../muoxi/src/server/typeclass.rs) |
| The `Registry` API | [`muoxi/src/server/registry.rs`](../muoxi/src/server/registry.rs) |
| The `WorldApi` facade | [`muoxi/src/server/world.rs`](../muoxi/src/server/world.rs) |
| Lock-expression evaluator | [`muoxi/src/server/locks.rs`](../muoxi/src/server/locks.rs) |
| DB schema | [`db/src/schema.rs`](../db/src/schema.rs) and the SQL in [`migrations/`](../migrations/) |
| Object / Attribute / Tag repos | [`db/src/objects/`](../db/src/objects/) |
| argon2 password hashing | [`muoxi/src/server/auth.rs`](../muoxi/src/server/auth.rs) |
| World seeding | [`muoxi/src/server/seed.rs`](../muoxi/src/server/seed.rs) |
| TCP server entry point | [`muoxi/src/server/main.rs`](../muoxi/src/server/main.rs) |
| Per-client `process()` lifecycle | [`muoxi/src/lib.rs`](../muoxi/src/lib.rs) |
| WebSocket bridge | [`muoxi/src/webserver/webserver.rs`](../muoxi/src/webserver/webserver.rs) |

## Conventions

The patterns the codebase follows:

**Rust toolchain.** Edition 2024, stable, MSRV 1.85. Pinned via
`rust-toolchain.toml` — don't disable it; local nightly may regress
against bleeding-edge deps.

**Tokio.** 1.x only. `tokio::prelude` doesn't exist any more; use
individual `AsyncReadExt` / `AsyncWriteExt` imports.
`tokio_stream::StreamExt` for `.next()` on `Framed`,
`futures_util::SinkExt` for `.send()`.

**Diesel.** 2.x. Query helpers take `&mut Conn`, never `&Conn`. Macros
are namespaced (`diesel::table!`, `diesel::insert_into`). Don't
hand-edit `db/src/schema.rs` — regenerate via `diesel migration run`
with `diesel.toml` pointing at it.

**Backend portability.** SQLite is the default. The schema stays
portable, so no Postgres-only types in core: no `BIGINT[]`, no `JSONB`,
no `LISTEN/NOTIFY`, no `to_tsvector`. Exactly one backend is active at
build time — the compile-error guard in
[`db/src/conn.rs`](../db/src/conn.rs) enforces it.

**Type safety.** No `as any`-style escape hatches.
`#![deny(missing_docs)]` on the `db` crate — every public item has a
docstring.

**Repo discipline.** Commands and hooks go through `WorldApi`. If
`WorldApi` doesn't expose what you need, add it there.

## Testing matrix

| You changed | Run |
| --- | --- |
| `db/src/schema.rs` or `migrations/` | `cargo test -p db --features db-sqlite` |
| `db/src/objects/` or `db/src/structures.rs` | same |
| `muoxi/src/server/registry.rs`, `typeclass.rs`, `commands/` | `cargo test -p muoxi --test registry` |
| `muoxi/src/server/auth.rs` | `cargo test -p muoxi --lib auth` |
| Anything in `muoxi/src/server/` (broad) | `cargo check --workspace && cargo clippy --workspace --no-deps` |
| Docker / deploy / `Dockerfile` | `docker compose build && docker compose up` then run through [getting-started.md](getting-started.md) |
| Postgres backend code paths | `cargo check -p db --no-default-features --features db-postgres` |

## Running the server during dev

`DEV_AUTOLOGIN` skips the auth state machine and drops new connections
straight into `Playing`:

```bash
DEV_AUTOLOGIN=1 cargo run --bin muoxi_server
# in another shell:
telnet 127.0.0.1 8000
```

For the web client:

```bash
cargo run --bin muoxi_web &
open http://localhost:8080
```

The web bridge reads its page out of `include_str!`, so changes to
`resources/web/index.html` need a rebuild to take effect.

## Adding a migration

1. Create `migrations/<YYYY-MM-DD-NNNNNN>_<descriptor>/`.
2. Write `up.sql` and `down.sql`. Use portable types only.
3. The migration is embedded into the binary at build time, applied on
   `DatabaseHandler::connect()`. No `diesel migration run` is needed at
   runtime.
4. Regenerate `db/src/schema.rs` via `diesel migration run` against a
   local DB, then check in the result.

## Adding a dependency

External deps are centralized in the workspace root `Cargo.toml`
`[workspace.dependencies]`. Member crates reference them as
`{ workspace = true }`.

To add one:

1. Add the version to root `Cargo.toml`.
2. Add `name = { workspace = true }` to the member that uses it.
3. Run `cargo update -p <name>` so the lockfile resolves.

The repo runs lean on purpose; deps are worth thinking about.

## Logging

The default filter is forced inside `main()` to `info,warn,error,test`.
Override per-run:

```bash
RUST_LOG=debug cargo run --bin muoxi_server
RUST_LOG=muoxi::server::states=trace,info cargo run --bin muoxi_server
```

Trace level on the state-machine module is useful when debugging
transitions.

## PR process

Issues first for non-trivial changes. Branch from `master`. Atomic
commits. Run the testing rows for what you touched. Push, open the PR,
explain the why in the description.

There's no automated CI yet; local checks are the gate.

## See also

- [extension-guide.md](extension-guide.md) — the public extension
  surface
- [architecture.md](architecture.md) — design rationale
- [roadmap.md](roadmap.md) — where the project is headed
- [CONTRIBUTING.md](../CONTRIBUTING.md) — top-level contributor guide
