# ![muoxi_logo][logo]

# MuOxi — a MUD/MU* framework in Rust

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

MuOxi is a framework for building [online multiplayer text games][wikimudpage]
(MUDs, MUSHes, MUCKs — the MU\* family). The framework provides the boring
parts — TCP listener, WebSocket bridge, login state machine, persistent object
model, attribute/tag bags, character/account binding — so downstream developers
can focus on the *world*: rooms, items, mobs, combat, magic, economy.

The project draws from [Evennia][evennia]'s philosophy (generic typed
objects, freeform attributes, hook-based extension) but is built around Rust's
async runtime and type system for the throughput and safety they bring.

```
                                              ┌────────────┐
   tt++ / telnet      ─tcp→   ┌──────────────┐│   redis    │ (transient
   browser / WS       ─http→  │ muoxi_server │└─────┬──────┘  session cache)
   muoxi_web bridge   ─ws──→  └──────┬───────┘      │
                                     │              ▼
                                     ▼        ┌────────────┐
                              ┌────────────┐  │  Diesel    │
                              │   engine   │←→│  SQLite /  │
                              │   (rust)   │  │  Postgres  │
                              └────────────┘  └────────────┘
```

## Status — actively in revival (May 2026)

This project went dormant for several years. As of May 2026 it's back in
active development and being shaped into a v0.1 framework release.

**What works today** (commit `47a2d4f` on `master`):

- TCP/telnet server on `127.0.0.1:8000` — connects, serves welcome banner,
  runs a per-client connection-state machine.
- WebSocket bridge with a built-in in-browser test client at
  `http://localhost:8080` (vanilla JS, no build step).
- Generic object/attribute/tag persistence layer (Evennia-style) backed by
  SQLite (default) or Postgres (opt-in).
- Redis-backed transient session cache.
- Single-binary Docker stack (`docker compose up`).
- Toolchain pinned to stable Rust 1.85; default build needs **zero system
  packages** (SQLite is bundled).

**What's still in progress** — see [Roadmap](#roadmap).

## Quick start

```bash
git clone https://github.com/duysqubix/MuOxi.git
cd MuOxi
docker compose up
```

Then connect any of three ways:

| Surface       | URL                           | Notes                                         |
| ------------- | ----------------------------- | --------------------------------------------- |
| Browser       | `http://localhost:8080`       | Loads a vanilla-JS WS test client             |
| Telnet / tt++ | `127.0.0.1:8000`              | `telnet 127.0.0.1 8000` or `tt++` `#session`  |
| WS client     | `ws://localhost:8080`         | `wscat -c ws://localhost:8080` and any sender |

If host ports 8000 / 8080 are taken:

```bash
MUOXI_SERVER_PORT=18000 MUOXI_WEB_PORT=18080 docker compose up
```

The first run creates `data/world.db` (SQLite) inside a named docker volume.

## Building from source

```bash
cargo build --workspace                 # SQLite default; zero system packages
cargo run --bin muoxi_server            # 127.0.0.1:8000 — login + game in one process
cargo run --bin muoxi_web               # ws://127.0.0.1:8080 → tcp 127.0.0.1:8000
```

Connect with `telnet 127.0.0.1 8000` or your favourite MUD client.

### Optional: Postgres backend

For deployments with multiple writers or replicas, opt into Postgres at compile
time:

```bash
sudo apt install libpq-dev              # or your platform's libpq package
cargo build --no-default-features --features db-postgres
DATABASE_URL=postgres://muoxi:muoxi@localhost/muoxi \
  cargo run --bin muoxi_server
```

The same migrations under `migrations/` apply to both backends.

### Optional: Redis

MuOxi caches per-connection socket state in Redis. The server boots without
Redis, but you'll see cache errors in the log. For local dev:

```bash
redis-server                            # default port 6379
REDIS_SERVER=redis://127.0.0.1 cargo run --bin muoxi_server
```

The supplied `docker compose` already wires Redis as a sidecar service.

## Architecture

MuOxi is built as a Cargo workspace with four member crates:

| Crate          | Role                                                                  |
| -------------- | --------------------------------------------------------------------- |
| `muoxi`        | App crate; ships `muoxi_server` (TCP) and `muoxi_web` (WS bridge)     |
| `db`           | Persistence + caching library: Diesel + Redis. SQLite default.        |
| `benchmarks`   | Standalone non-Criterion benchmark harness                            |
| `tester`       | Sandbox binary for manual exploration (NOT a test suite)              |

### Persistence model

The DB has two layers:

1. **Canonical store** — SQLite (or Postgres) accessed via Diesel ORM.
2. **Transient cache** — Redis holds per-session ephemeral state (socket
   address, UID). Sessions survive Redis going down; nothing persistent is
   ever written there.

### Object model

Every in-world entity (rooms, items, mobs, exits, characters, NPCs, plus any
type a downstream framework user defines) is a row in a single `objects`
table, discriminated by a `type_key` text column. Per-entity gameplay state
lives in two satellite tables:

- `object_attributes` — freeform key→JSON-text bag (set HP, set loot, set
  description, …)
- `object_tags` — labels with optional category, used for grouping and lookup
  (find all rooms tagged `safe-zone`)

This means **adding a new in-world type does not require a schema migration**.
You pick a `type_key`, you call `db.objects.create(conn, "weapon", "Sword", room.uid)`,
and you stash type-specific state in `db.attributes.set(uid, "damage", json!(7))`.

Login identity (accounts, password hashes, email) lives in its own typed
`accounts` table — it's fundamentally different from in-world state.

### Topology

For v0.1, `muoxi_server` is a single Tokio process holding the TCP listener,
login state machine, and game logic. `muoxi_web` is a thin protocol adapter
that bridges WebSocket clients to the same TCP backend.

The portal/server split (separate proxy + engine processes with a framed
protocol enabling hot-reload) is on the v0.2 roadmap.

## Roadmap

The path to v0.1 is broken into six axis-plans under
[`docs/superpowers/plans/`](docs/superpowers/plans/):

| Plan | Topic                                            | Status         |
| ---- | ------------------------------------------------ | -------------- |
| 1    | SQLite migration + drop JSON/watchdog            | ✅ done        |
| 2    | Topology collapse → unified `muoxi_server`       | ✅ done        |
| 3    | Generic Object/Attribute/Tag model               | ✅ done        |
| 4    | Command + Hook + TypeClass registry              | ⏳ next        |
| 5    | Persistent scheduler / scripts                   | ⏳             |
| 6    | Full auth state machine (argon2 + login flow)    | ⏳             |

Start with the
[MASTER-PLAN](docs/superpowers/plans/2026-05-07-MASTER-PLAN.md) — it has a
"resume point" header that any agent or human picking up the work can use to
get oriented quickly.

## For framework users

The end goal is a framework where you can:

```rust
// (hypothetical post-Plan-4 API — not yet final)
use muoxi::prelude::*;

#[muoxi::typeclass(key = "weapon")]
struct Weapon;

impl Weapon {
    #[hook(at_create)]
    fn on_create(obj: &mut Object) -> anyhow::Result<()> {
        obj.attributes.set("damage", json!(7))?;
        Ok(())
    }
}

#[muoxi::command(name = "swing")]
async fn cmd_swing(client: &mut Client, args: Vec<String>) -> CommandResult<()> {
    // ...
}

fn main() {
    Registry::new()
        .register_typeclass::<Weapon>()
        .register_command::<CmdSwing>()
        .run();
}
```

Plan 4 lands this surface. Until then, downstream code reaches into the lower
layers (`muoxi/src/server/`, `db/src/objects/`) directly. See
[`AGENTS.md`](AGENTS.md) for the current code map.

## Development

### Tests

```bash
cargo test -p db --features db-sqlite     # 5 integration tests, in-memory
```

The `db` crate carries integration tests under `db/tests/`. Tests use the
default SQLite backend and require no system dependencies.

### Type-check both backends

```bash
cargo check --workspace                                              # SQLite (default)
cargo check -p db --no-default-features --features db-postgres       # Postgres
```

### Lint

```bash
cargo clippy --workspace --no-deps
```

### Knowledge bases

Every non-trivial directory has an `AGENTS.md` documenting layout,
conventions, and anti-patterns specific to that subsystem:

- [`AGENTS.md`](AGENTS.md) — repo-wide overview, code map, env vars
- [`db/AGENTS.md`](db/AGENTS.md) — persistence layer, schema, repos
- [`muoxi/AGENTS.md`](muoxi/AGENTS.md) — app crate, binaries, conventions
- [`muoxi/src/server/AGENTS.md`](muoxi/src/server/AGENTS.md) — server module, state machine
- [`benchmarks/AGENTS.md`](benchmarks/AGENTS.md) — benchmark harness
- [`tester/AGENTS.md`](tester/AGENTS.md) — sandbox binary

These files are designed for AI agents and new contributors alike.

## Contributing

PRs welcome. See [`CONTRIBUTING.md`](CONTRIBUTING.md) for setup, conventions,
and how to find a good first issue (or pick a Plan-4-onward task that fits
your interests).

Reach out on [discord][discord] for design conversations.

## License

GPL v3 — see [`LICENSE`](LICENSE).

[logo]:        .media/cog.png
[wikimudpage]: https://en.wikipedia.org/wiki/MUD
[evennia]:     https://www.evennia.com/
[discord]:     https://discord.gg/H6Sh3CJ
