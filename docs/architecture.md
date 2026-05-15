# Architecture

This document explains how MuOxi is structured, what it provides, what it
deliberately doesn't, and why.

## Design philosophy

MuOxi is a **framework**, not a game. It ships the parts that every MUD
needs — sockets, login, persistence, command dispatch, world state — and
gets out of your way for the parts that are *your* MUD: combat, magic,
crafting, economy, factions, plot, voice.

The closest spiritual ancestor is [Evennia](https://www.evennia.com/) (Python).
MuOxi borrows three ideas from Evennia:

1. **Everything in the world is an "Object"** with a `type_key` that tells
   the engine what kind of thing it is.
2. **Per-object freeform attributes** (key → JSON value) so adding a new
   field doesn't require a schema migration.
3. **Tags** for grouping and lookup ("all rooms marked `safe-zone`", "all
   objects with the `pvp` permission").

What MuOxi adds: Rust's type system, Tokio's async runtime, and Diesel's
typed SQL. The trade is that you write your game in Rust — a higher floor
than Python, but with throughput and safety that scales further.

## What MuOxi provides

| Layer | Provides |
| --- | --- |
| Sockets | TCP listener; line-delimited protocol; a WebSocket bridge with an in-browser test client |
| Connection lifecycle | Per-client state machine, session UID, Redis-backed transient session cache |
| Authentication | argon2id password hashing, account creation flow, character select/create, lifecycle hooks (`at_login`, `at_disconnect`) |
| Persistence | SQLite by default (zero system deps), Postgres opt-in via `--features db-postgres`; embedded migrations |
| Object model | Generic `Object` table + `ObjectAttribute` (JSON-text bag) + `ObjectTag` (categorized labels) + `CharacterAccount` link |
| Extension surface | `Registry` of `TypeClass`es, commands, and hooks; `WorldApi` facade for DB access; lock-expression DSL for command permissions |
| Built-in command set | `look`, `say`, `quit`, `who` (replace or extend) |
| Built-in TypeClasses | `character`, `room`, `item`, `exit`, `mob` (extend or define your own) |
| World seeding | Idempotent `seed_world()` that ensures a starting room exists; replace with your own |
| Developer ergonomics | `DEV_AUTOLOGIN` mode skips auth for fast iteration; embedded migrations mean fresh installs Just Work |

## What MuOxi does NOT provide

These are explicitly out of scope for the framework. Building them is YOUR
job (or a future opt-in module).

- A combat system
- A magic / spell / skill system
- An economy or currency
- A quest engine
- A specific MUD theme or content beyond the placeholder "Limbo" starting room
- A default permission/role hierarchy beyond the tiny lock DSL
- A world-building tool / OLC system (in-game room editor)
- Anti-cheat, rate-limiting, abuse mitigation
- Localization / i18n
- Persistent player chat history / logging

This is deliberate. Each of those decisions reflects taste — a horror MUD's
combat is nothing like a sci-fi MUD's combat. MuOxi gives you the
extension points to wire any of those in; it doesn't pretend to know what
they should look like.

## Topology

For v0.1, the server is a single Tokio process:

```
                                          ┌────────────┐
   tt++ / telnet      ─tcp→  ┌───────────┐│   redis    │ (transient
   browser / WS       ─http→ │muoxi_server│└────┬───────┘  session cache,
   muoxi_web bridge   ─ws──→ └─────┬─────┘     │           per-conn state)
                                   │ owns      │
            ┌──────────────────────┼───────────┘
            │                      │
            ▼                      ▼
      ┌─────────┐         ┌─────────────┐
      │ Registry│         │ DatabaseHandler │
      └────┬────┘         │   (Diesel)      │
           │              └────────┬────────┘
           │                       │
   types/commands/hooks    ┌───────┴────────┐
                           │ SQLite (default) │
                           │ or Postgres      │
                           └──────────────────┘
```

The portal/server split — separate proxy and engine processes communicating
via a framed protocol, enabling hot-reload — is on the v0.2 roadmap. For
v0.1, one process keeps the surface area smaller and the demo path shorter.

### `muoxi_server`

Owns:

- The TCP listener on `PROXY_ADDR` (default `127.0.0.1:8000`)
- The connection-state machine (`AwaitingName` → `AwaitingPassword` → `MainMenu` → `Playing`)
- The `Registry` (TypeClasses, commands, hooks) — built once at startup
- The `WorldApi` (DB facade wrapping `DatabaseHandler`)
- Per-client `Server.clients` map for the in-memory roster

Implementation: [`muoxi/src/server/main.rs`](../muoxi/src/server/main.rs) for the
binary entry; the actual library surface lives in
[`muoxi/src/lib.rs`](../muoxi/src/lib.rs) and the modules under
[`muoxi/src/server/`](../muoxi/src/server/).

### `muoxi_web`

A WebSocket bridge. Listens on `WEB_ADDR` (default `127.0.0.1:8080`).
Per-WS-client, it opens a fresh outbound TCP connection to `PROXY_ADDR` and
relays line-oriented text frames in both directions.

Plain HTTP GET to the same port returns the bundled in-browser test client
(`resources/web/index.html`). This is a vanilla-JS WebSocket terminal —
useful for trying MuOxi from a browser without installing a MUD client.

Implementation: [`muoxi/src/webserver/webserver.rs`](../muoxi/src/webserver/webserver.rs).

### Persistence layers

Two stores, with deliberately different responsibilities:

1. **Canonical state — Diesel + SQLite/Postgres**
   - Schema lives in [`db/src/schema.rs`](../db/src/schema.rs)
   - Migrations under [`migrations/`](../migrations/) are embedded into the
     binary at compile time and applied on `DatabaseHandler::connect()`
   - Default backend is SQLite via bundled `libsqlite3-sys` — **zero
     system packages required for a default build**
   - Postgres opt-in: `cargo build --no-default-features --features db-postgres`
     (requires `libpq-dev`)

2. **Transient state — Redis**
   - Per-session ephemera: socket address, session UID
   - The server boots without Redis (you'll see cache errors in the log)
   - Sessions survive Redis going down; no persistent data lives there

The split matters: if Redis disappears, you lose connection metadata but no
game state.

## Object model

Every in-world entity is a row in the `objects` table:

```
objects
  uid           BIGINT PK
  type_key      TEXT       ── "character", "room", "item", "exit", "mob", ...
  name          TEXT
  location_uid  BIGINT?    ── self-FK: what container is this in
  created_at    BIGINT     ── unix epoch seconds
```

The `type_key` discriminator is what makes adding new in-world types
**not require a schema change**. You pick a `type_key`, call
`world.create_object("dragon", "Vermithrax", Some(room.uid))`, and that's it.

Per-entity state lives in two satellite tables:

```
object_attributes
  object_uid    BIGINT FK → objects(uid) ON DELETE CASCADE
  key           TEXT
  value         TEXT        ── JSON-encoded; serde_json::Value at the Rust boundary
  PRIMARY KEY (object_uid, key)

object_tags
  object_uid    BIGINT FK → objects(uid) ON DELETE CASCADE
  key           TEXT
  category      TEXT
  PRIMARY KEY (object_uid, key, category)
```

`object_attributes` is the freeform JSON bag. `object_tags` is for
fast lookups across objects ("find all `safe-zone` rooms") and is also
the mechanism the lock DSL uses for `perm(X)` checks (`(X, "permission")`
tag on the actor's character).

Login identity lives in its own typed `accounts` table — it's fundamentally
different from in-world state. Characters belong to accounts via the
`character_accounts` link table.

```
accounts
  uid PK
  name UNIQUE
  password_hash      ── argon2id PHC string
  email
  created_at

character_accounts
  object_uid PK → objects(uid) ON DELETE CASCADE
  account_uid    → accounts(uid) ON DELETE CASCADE
  ordinal        ── 0-indexed slot in this account's character list
```

## Connection state machine

```
       ┌──────────────┐
       │ AwaitingName │◄─────────────── (bad password / no account / lost session)
       └───┬──────────┘
   new │   │ existing-account
       ▼   ▼
┌──────────┐  ┌────────────┐
│AwNewName │  │AwPassword  │── bad password ──► AwaitingName
└────┬─────┘  └─────┬──────┘
     ▼              │ argon2 verify ok
┌──────────┐        │ + at_login hook fires
│AwNewPass │        ▼
└────┬─────┘  ┌──────────┐
     ▼        │ MainMenu │
┌──────────┐  └────┬─────┘
│ConfNewPwd│       │ select N (existing) / new <name> (new char)
└────┬─────┘       ▼
     │        ┌─────────┐
     └────────► Playing │── quit ──► Quit ── at_disconnect hook fires
              └────┬────┘
                   │ each line dispatched via
                   ▼
              Registry::resolve_command → lock check → Command::execute_cmd
```

`Client.auth_buffer` carries the partial inputs (pending name, first
password attempt) across the transitions. `Client.account_uid` is `Some`
after successful auth or new-account creation. `Client.character_uid` is
`Some` after character select or character creation.

The `DEV_AUTOLOGIN=1` env var **bypasses** the state machine entirely:
new connections jump straight to `Playing` as a throwaway "Dev" character
in the seeded starter room. This is a developer convenience, not a
production feature.

## Command dispatch

When a client is in the `Playing` state, each line of input is sent through
[`muoxi/src/server/cmds.rs::dispatch`](../muoxi/src/server/cmds.rs):

1. Take the first whitespace-delimited token, case-insensitive.
2. `Registry::resolve_command(token)` — look up by name or alias.
3. If found, evaluate `cmd.lock()` against the actor via `locks::check`.
   On deny: send "You can't do that." and stop.
4. Build a `CommandContext` carrying the client, the registry, the
   `WorldApi`, and the rest of the input as `args`.
5. Run `cmd.execute_cmd(ctx)`. Errors get prefixed and sent back.

The lock DSL currently supports three forms: `all()` (default; always
allow), `false` (never allow), `perm(NAME)` (actor must have the
`(NAME, "permission")` tag).

## Hooks

Hooks are lifecycle event listeners that downstream MUDs implement to react
to engine-level events. The trait is in
[`muoxi/src/server/hooks.rs`](../muoxi/src/server/hooks.rs) and has seven
methods, all default-no-op:

| Method | When | Cancelable? |
| --- | --- | --- |
| `at_login(account_uid)` | After successful auth | No |
| `at_disconnect(account_uid)` | Before cleanup (only if logged in) | No |
| `at_object_created(obj)` | After `Object` insertion | No |
| `at_pre_destroy(obj)` | Before object deletion | **Yes** — `Err` cancels |
| `at_pre_move(obj, src, dst)` | Before location change | **Yes** — `Err` cancels |
| `at_post_move(obj, src, dst)` | After location change | No |
| `at_say(speaker, message)` | After `say` command | **Yes** — `Err` suppresses delivery |

**As of v0.1, only `at_login` and `at_disconnect` are actually fired by the
engine.** The other five are extension points that downstream MUDs can
implement, but the engine doesn't yet emit them. Wiring them is on the
v0.2 roadmap.

## Code map

The full repo layout:

```
.
├── muoxi/                      # application crate
│   ├── src/
│   │   ├── lib.rs              # library half: process(), send/get helpers, SessionConfig
│   │   ├── server/
│   │   │   ├── main.rs         # muoxi_server binary entrypoint
│   │   │   ├── auth.rs         # argon2 hash/verify, AuthBuffer, validators
│   │   │   ├── cmds.rs         # command dispatcher (resolve → lock → execute)
│   │   │   ├── commands/       # built-in commands (look, say, quit, who)
│   │   │   ├── comms.rs        # Client + Server connection state
│   │   │   ├── engine.rs       # pass-through extension point for pre/post-input
│   │   │   ├── hooks.rs        # Hook trait + Hooks collection
│   │   │   ├── locks.rs        # lock-expression evaluator
│   │   │   ├── prelude.rs      # Command trait + CommandContext
│   │   │   ├── registry.rs     # Registry (types, commands, hooks)
│   │   │   ├── seed.rs         # seed_world (default starter room)
│   │   │   ├── states.rs       # ConnStates enum + transition machine
│   │   │   ├── typeclass.rs    # TypeClass trait + 5 built-in types
│   │   │   └── world.rs        # WorldApi (DB facade)
│   │   └── webserver/
│   │       └── webserver.rs    # muoxi_web binary (WS bridge + browser test client)
│   └── tests/
│       └── registry.rs         # registry/typeclass smoke tests
├── db/                         # persistence library
│   ├── src/
│   │   ├── lib.rs              # DatabaseHandler facade; embedded migrations
│   │   ├── conn.rs             # backend selection (sqlite/postgres feature)
│   │   ├── schema.rs           # generated; do not hand-edit
│   │   ├── structures.rs       # Account model + DatabaseHandlerExt trait
│   │   ├── objects/            # Object, ObjectAttribute, ObjectTag, CharacterAccount + repos
│   │   ├── cache.rs            # Redis Connection factory
│   │   ├── cache_structures/   # Cachable trait + CacheSocket
│   │   └── utils.rs            # UID type, gen_uid()
│   └── tests/                  # in-memory SQLite integration tests
├── examples/extension/         # downstream MUD embedding demo
├── migrations/                 # Diesel migrations, embedded at compile time
├── resources/
│   ├── welcome.txt             # login banner served at connect
│   └── web/index.html          # browser WS test client
├── data/                       # SQLite world.db lands here at runtime (gitignored)
└── docs/                       # you are here
```

## Read next

- Concrete walkthrough of running it and seeing it work →
  [getting-started.md](getting-started.md)
- Every extension point in full detail →
  [extension-guide.md](extension-guide.md)
- What's on the roadmap and what's deliberately out of scope →
  [roadmap.md](roadmap.md)
