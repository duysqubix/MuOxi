# Architecture

This document explains how MuOxi is structured and why. If you want to
run it first and read this afterwards, that's a fine order — try
[getting-started.md](getting-started.md).

## Design philosophy

MuOxi is a framework, not a game. It ships infrastructure — sockets,
login, persistence, command dispatch, hooks — and gets out of the way
for everything else.

The closest spiritual ancestor is [Evennia](https://www.evennia.com/),
which brought three ideas worth borrowing:

1. Every in-world thing is a generic `Object` with a `type_key` that
   says what kind of thing it is.
2. Per-object freeform attributes — a JSON bag keyed by string — so
   adding a new field doesn't mean a schema migration.
3. Tags for grouping and lookup ("all rooms tagged `safe-zone`").

MuOxi takes those ideas into Rust, with Diesel for typed SQL and Tokio
for an async runtime. You write your game in Rust; the floor is higher
than Python's, but the throughput, type safety, and ergonomics go
further when the project grows.

## Topology

`muoxi_server` is a single Tokio process. It owns the TCP listener, the
connection-state machine, the in-process command dispatcher, the
Registry of extension points, and the database connection.

`muoxi_web` is a thin protocol adapter on its own port. Plain HTTP GET
returns a bundled browser test client; a WebSocket upgrade opens a
fresh outbound TCP connection to `muoxi_server` and bridges line-oriented
text frames in both directions.

```
                                          ┌────────────┐
   tt++ / telnet      ─tcp→  ┌───────────┐│   redis    │
   browser / WS       ─http→ │muoxi_server│└────┬───────┘
   muoxi_web bridge   ─ws──→ └─────┬─────┘     │
                                   │           ▼
            ┌──────────────────────┼─────────────────┐
            ▼                      ▼                 ▼
      ┌─────────┐         ┌────────────────┐  ┌──────────────┐
      │ Registry│         │DatabaseHandler │  │ Redis cache  │
      └────┬────┘         │   (Diesel)     │  │ (per-session │
           │              └────────┬───────┘  │  ephemera)   │
   types/commands/hooks            │          └──────────────┘
                                   ▼
                          ┌──────────────────┐
                          │ SQLite (default) │
                          │ or Postgres      │
                          └──────────────────┘
```

A separate proxy and engine, talking over a framed protocol, is on the
horizon — see [roadmap.md](roadmap.md). For now, one process keeps the
surface area small.

### Persistence layers

Two stores with different jobs.

The canonical store is SQLite or Postgres via Diesel. Schema lives in
[`db/src/schema.rs`](../db/src/schema.rs) and migrations under
[`migrations/`](../migrations/) are embedded into the binary at compile
time, applied on `DatabaseHandler::connect()`. SQLite is the default
and needs no system packages; Postgres is opt-in.

The transient store is Redis. It holds per-session ephemera — socket
address, session UID. The server boots without Redis (you'll see cache
errors in the log, but the game keeps running), and no persistent data
ever lives there.

The split matters: if Redis disappears, you lose connection metadata
but no game state.

## Object model

Every in-world entity is a row in the `objects` table:

```
objects
  uid           BIGINT PK
  type_key      TEXT       — "character", "room", "item", "exit", "mob", …
  name          TEXT
  location_uid  BIGINT?    — self-FK: what container holds this
  created_at    BIGINT
```

The `type_key` discriminator is what makes new in-world types cheap. You
pick a key, call `world.create_object("dragon", "Vermithrax", Some(room.uid))`,
and you're done — no schema change.

Per-entity state lives in two satellite tables: `object_attributes` for
the freeform JSON bag and `object_tags` for searchable `(key, category)`
labels. Both cascade on object delete.

Login identity lives in its own typed `accounts` table — it's
fundamentally different from in-world state. Characters belong to
accounts via the `character_accounts` link table.

The full schema is in [`db/src/schema.rs`](../db/src/schema.rs).

## Connection state machine

```
       ┌──────────────┐
       │ AwaitingName │◄────── (bad password / no account / lost session)
       └───┬──────────┘
   new │   │ existing
       ▼   ▼
┌──────────┐  ┌────────────┐
│AwNewName │  │AwPassword  │── bad ──► AwaitingName
└────┬─────┘  └─────┬──────┘
     ▼              │ argon2 verify ok
┌──────────┐        │ + at_login hook fires
│AwNewPass │        ▼
└────┬─────┘  ┌──────────┐
     ▼        │ MainMenu │
┌──────────┐  └────┬─────┘
│ConfNewPwd│       │ select N / new <name>
└────┬─────┘       ▼
     │        ┌─────────┐
     └────────► Playing │── quit ──► Quit ── at_disconnect hook fires
              └────┬────┘
                   │ each line through cmds::dispatch
                   ▼
              Registry::resolve_command → lock check → Command::execute_cmd
```

`Client.auth_buffer` carries the partial inputs across transitions.
`Client.account_uid` is set after auth or new-account creation.
`Client.character_uid` is set after character select or creation.

The `DEV_AUTOLOGIN=1` env var bypasses the state machine entirely:
connections jump straight to `Playing` as a throwaway "Dev" character.
A developer convenience, not a production feature.

## Command dispatch

When a client is in `Playing`, each line goes through
[`cmds.rs::dispatch`](../muoxi/src/server/cmds.rs):

1. Take the first whitespace-delimited token, lowercase.
2. `Registry::resolve_command(token)` — look up by name or alias.
3. Evaluate `cmd.lock()` against the actor via `locks::check`.
4. Build a `CommandContext` with the client, registry, world, and rest
   of the input as `args`.
5. Run `cmd.execute_cmd(ctx)`.

The lock language is small: `all()`, `false`, and `perm(NAME)` (the
actor must carry a `(NAME, "permission")` tag).

## Hooks

Hooks are lifecycle event listeners that downstream MUDs implement to
react to engine events. The trait — in
[`hooks.rs`](../muoxi/src/server/hooks.rs) — has seven methods, all
default-no-op: `at_login`, `at_disconnect`, `at_object_created`,
`at_pre_destroy`, `at_pre_move`, `at_post_move`, `at_say`. The
`_pre_*` ones are cancelable: returning `Err` aborts the in-flight
action.

Emission is being wired in over time. See [roadmap.md](roadmap.md) for
the current status.

## Code map

```
.
├── muoxi/                      # application crate
│   ├── src/
│   │   ├── lib.rs              # library half: process(), helpers, SessionConfig
│   │   ├── server/
│   │   │   ├── main.rs         # muoxi_server binary entrypoint
│   │   │   ├── auth.rs         # argon2 hash/verify, AuthBuffer, validators
│   │   │   ├── cmds.rs         # command dispatcher
│   │   │   ├── commands/       # built-in commands
│   │   │   ├── comms.rs        # Client + Server connection state
│   │   │   ├── engine.rs       # pass-through extension point
│   │   │   ├── hooks.rs        # Hook trait + Hooks collection
│   │   │   ├── locks.rs        # lock-expression evaluator
│   │   │   ├── prelude.rs      # Command trait + CommandContext
│   │   │   ├── registry.rs     # Registry
│   │   │   ├── seed.rs         # seed_world
│   │   │   ├── states.rs       # ConnStates state machine
│   │   │   ├── typeclass.rs    # TypeClass trait + built-ins
│   │   │   └── world.rs        # WorldApi DB facade
│   │   └── webserver/
│   │       └── webserver.rs    # muoxi_web binary
│   └── tests/registry.rs
├── db/                         # persistence library
│   ├── src/
│   │   ├── lib.rs              # DatabaseHandler; embedded migrations
│   │   ├── conn.rs             # backend selection
│   │   ├── schema.rs           # generated; not hand-edited
│   │   ├── structures.rs       # Account model + DatabaseHandlerExt
│   │   ├── objects/            # Object + Attribute + Tag + CharacterAccount repos
│   │   ├── cache.rs            # Redis Connection factory
│   │   └── cache_structures/   # Cachable + CacheSocket
│   └── tests/
├── examples/extension/         # downstream MUD embedding demo
├── migrations/                 # Diesel migrations (embedded)
├── resources/                  # welcome.txt + web/index.html
├── data/                       # SQLite world.db at runtime (gitignored)
└── docs/                       # you are here
```

## Read next

- [extension-guide.md](extension-guide.md) — every extension point in
  detail with worked examples
- [world-building.md](world-building.md) — how to populate the world
- [roadmap.md](roadmap.md) — where the project is headed
