# Development

For hacking on MuOxi itself — the framework. If you're using MuOxi to
build *a* MUD, read [getting-started.md](getting-started.md) and
[extension-guide.md](extension-guide.md) instead.

## Local setup

```bash
git clone https://github.com/duysqubix/MuOxi.git
cd MuOxi
cargo build --workspace
```

`rustup` will fetch the toolchain version pinned in
[`rust-toolchain.toml`](../rust-toolchain.toml) automatically. No system
packages required for the default SQLite build.

## Verify the tree is healthy

```bash
cargo check --workspace
cargo test -p db --features db-sqlite
cargo test -p muoxi --test registry
cargo test -p muoxi --lib auth
cargo clippy --workspace --no-deps
```

If those all pass, you have a clean baseline.

## The two crates

- **`db/`** — persistence library. Diesel + Redis. Feature-gated backend
  (SQLite default, Postgres opt-in). No server-side code lives here.
- **`muoxi/`** — application crate. Two binaries (`muoxi_server` and
  `muoxi_web`) plus a library half that exports the registry/typeclass/hook/
  WorldApi/etc. for downstream consumers and tests.

The `muoxi` crate's library half lives in
[`muoxi/src/lib.rs`](../muoxi/src/lib.rs) and uses `#[path]` attributes to
pull modules from `muoxi/src/server/`. This lets the binary at
`muoxi/src/server/main.rs` and external crates both access the same
module tree.

## Subsystem-level invariants

Each crate / submodule has an `AGENTS.md` documenting its local
conventions and anti-patterns. **Read these before changing the
subsystem in question.**

- [Root `AGENTS.md`](../AGENTS.md) — repo-wide code map
- [`db/AGENTS.md`](../db/AGENTS.md) — Diesel patterns, schema rules,
  backend feature gating, Redis cache conventions
- [`muoxi/AGENTS.md`](../muoxi/AGENTS.md) — binary layout, lib+bin split,
  Tokio import gotchas, dependency choices
- [`muoxi/src/server/AGENTS.md`](../muoxi/src/server/AGENTS.md) — state
  machine, command dispatch, hook firing, lock evaluator

## Where the work happens

| You want to change | Look in |
| --- | --- |
| The connection-state machine | [`muoxi/src/server/states.rs`](../muoxi/src/server/states.rs) |
| Command dispatch (resolve → lock → execute) | [`muoxi/src/server/cmds.rs`](../muoxi/src/server/cmds.rs) |
| The `Command` / `CommandContext` trait shape | [`muoxi/src/server/prelude.rs`](../muoxi/src/server/prelude.rs) |
| The `Hook` trait or firing semantics | [`muoxi/src/server/hooks.rs`](../muoxi/src/server/hooks.rs) |
| The `TypeClass` trait or built-in types | [`muoxi/src/server/typeclass.rs`](../muoxi/src/server/typeclass.rs) |
| The `Registry` API | [`muoxi/src/server/registry.rs`](../muoxi/src/server/registry.rs) |
| The `WorldApi` facade | [`muoxi/src/server/world.rs`](../muoxi/src/server/world.rs) |
| Lock-expression evaluator | [`muoxi/src/server/locks.rs`](../muoxi/src/server/locks.rs) |
| The DB schema | [`db/src/schema.rs`](../db/src/schema.rs) + the SQL in [`migrations/`](../migrations/) |
| The `Object`/`Attribute`/`Tag` repos | [`db/src/objects/`](../db/src/objects/) |
| argon2 password hashing | [`muoxi/src/server/auth.rs`](../muoxi/src/server/auth.rs) |
| World seeding | [`muoxi/src/server/seed.rs`](../muoxi/src/server/seed.rs) |
| TCP server entry point + listener loop | [`muoxi/src/server/main.rs`](../muoxi/src/server/main.rs) |
| Per-client `process()` lifecycle | [`muoxi/src/lib.rs`](../muoxi/src/lib.rs) |
| WebSocket bridge | [`muoxi/src/webserver/webserver.rs`](../muoxi/src/webserver/webserver.rs) |

## Conventions

Tightest first.

### Rust toolchain & MSRV
- Edition 2024 everywhere. MSRV 1.85.
- Pinned via `rust-toolchain.toml`. Don't disable it locally — local
  nightly may regress against bleeding-edge deps.

### Tokio
- Tokio 1.x only. **No `tokio::prelude`** — that's Tokio 0.x. Use the
  individual `AsyncReadExt` / `AsyncWriteExt` imports.
- `tokio_stream::StreamExt` for `.next()` on `Framed`.
- `futures_util::SinkExt` for `.send()` on `Framed`.

### Diesel
- Diesel 2.x. Every query helper takes `&mut Conn`, never `&Conn`.
- Namespaced macros only (`diesel::table!`, `diesel::insert_into`).
  **No `#[macro_use] extern crate diesel;`**.
- Don't hand-edit `db/src/schema.rs`. Regenerate via `diesel migration run`
  with `diesel.toml` pointing at it.

### Backend portability
- SQLite is the default. Postgres-only types are banned in the core
  schema: no `BIGINT[]`, no `JSONB`, no `LISTEN/NOTIFY`, no `to_tsvector`,
  no `ON UPDATE CURRENT_TIMESTAMP`.
- Exactly one backend must be active. The compile-error guard in
  [`db/src/conn.rs`](../db/src/conn.rs) enforces this.

### Type safety
- **No `as any`-style escape hatches**: no `as any`, no `@ts-ignore`,
  no `#[allow(unused)]` to silence real issues. Fix it.
- `#![deny(missing_docs)]` on the `db` crate. Every public item gets a
  docstring.

### Repo discipline
- Commands and hooks never touch Diesel directly. Go through `WorldApi`.
  If `WorldApi` doesn't expose the method you need, add it there.
- The `json/` directory is for import/export payloads only — **never**
  re-introduce it as canonical state. The DB is the single source of
  truth.

### Commit messages
- Conventional-Commits-ish prefixes: `feat(scope): ...`, `fix(scope): ...`,
  `refactor(scope): ...`, `docs(scope): ...`, `test(scope): ...`,
  `chore(scope): ...`.
- Body explains the *why* in 1-3 short paragraphs.
- Atomic commits — each commit should be a single logical change that
  passes `cargo check --workspace`.

## Testing matrix

| You changed | Run |
| --- | --- |
| `db/src/schema.rs` or `migrations/` | `cargo test -p db --features db-sqlite` |
| `db/src/objects/` or `db/src/structures.rs` | same |
| `muoxi/src/server/registry.rs`, `typeclass.rs`, `commands/` | `cargo test -p muoxi --test registry` |
| `muoxi/src/server/auth.rs` | `cargo test -p muoxi --lib auth` |
| Anything in `muoxi/src/server/` (broad) | `cargo check --workspace && cargo clippy --workspace --no-deps` |
| Docker / deploy / `Dockerfile` | `docker compose build && docker compose up` then run through [getting-started.md](getting-started.md)'s walkthrough |
| Postgres backend code paths | `cargo check -p db --no-default-features --features db-postgres` |

## Running the server locally during dev

Iterate quickly with `DEV_AUTOLOGIN`:

```bash
DEV_AUTOLOGIN=1 cargo run --bin muoxi_server
# in another shell:
telnet 127.0.0.1 8000
```

You'll skip the auth state machine and land in `Playing` immediately.
Iterate on commands / hooks / typeclasses without typing credentials each
restart.

For the web client:

```bash
cargo run --bin muoxi_web &
open http://localhost:8080
```

The web bridge reads the page out of compiled `include_str!` data, so you
need to rebuild `muoxi_web` to see changes to `resources/web/index.html`.

## Adding a new migration

1. Create a new directory under `migrations/` named
   `<YYYY-MM-DD-NNNNNN>_<descriptor>/`.
2. Write `up.sql` (and `down.sql` for reversibility).
3. **Use portable types only** — see "Backend portability" above.
4. The migration is embedded into the binary at build time via
   `embed_migrations!` in [`db/src/lib.rs`](../db/src/lib.rs). No
   `diesel migration run` invocation needed at runtime.
5. Regenerate `db/src/schema.rs` via `diesel migration run` against a
   local DB, then check in the result.

## Adding a new dependency

Workspace dependency policy: every external dep is centralized in the
top-level `Cargo.toml` `[workspace.dependencies]`. Member crates
reference it via `{ workspace = true }`.

If you're adding a dep:

1. Add the version to root `Cargo.toml` `[workspace.dependencies]`.
2. Add the member-crate line `name = { workspace = true }`.
3. **Run `cargo update -p <name>`** so the lockfile reflects the
   resolved version.
4. Don't add deps casually. The repo runs lean on purpose.

## Logging during development

The default filter is `info,warn,error,test` (forced inside `main()`).
Override per-run:

```bash
RUST_LOG=debug cargo run --bin muoxi_server
RUST_LOG=muoxi::server::states=trace,info cargo run --bin muoxi_server
```

The trace-level log filter is useful when debugging state-machine
transitions.

## Where to find work

See the [roadmap](roadmap.md). The "Active work" section lists concrete
v0.1.x and v0.2 items.

Smaller starter tasks that don't fit the formal roadmap:

- The `who` command currently lists all character objects in the world,
  not just connected ones. Threading `Arc<Mutex<Server>>` through
  `CommandContext` would fix this.
- The hook-firing closure in `states.rs::AwaitingPassword` is awkward
  because `Hooks::emit` takes an `FnMut`. A redesign that lets the closure
  borrow `HookContext` once instead of re-constructing it per invocation
  would be cleaner.
- The lock DSL currently denies on parse failure. A "log on unknown
  expression" mode would help downstream MUDs catch typos.

## PR process

1. Open an issue first for non-trivial changes.
2. Branch from `master`.
3. Atomic commits — see "Commit messages" above.
4. Run the relevant rows from the testing matrix.
5. Push and open a PR. The PR description should explain *why* in
   addition to *what*.

There's no automated CI configured yet. Treat the local checks as the
gate.

## See also

- [extension-guide.md](extension-guide.md) — the public extension surface
- [architecture.md](architecture.md) — design rationale
- [roadmap.md](roadmap.md) — what's coming
- [CONTRIBUTING.md](../CONTRIBUTING.md) — top-level contributor guide
