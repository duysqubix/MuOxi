# server module

The unified MuOxi server. Owns the TCP listener, per-client session state,
the connection-state machine, and the registry-driven command dispatch.
Also the framework's library half — `muoxi/src/lib.rs` re-exports these
modules under `#[path]` attributes so downstream MUDs can depend on
`muoxi` as a crate.

## Files

| File | Role |
| --- | --- |
| `main.rs` | Binary entry. Constructs `WorldApi` + `Registry`, registers built-ins, spawns `process()` per accepted connection. |
| `comms.rs` | `Server` (`Arc<Mutex<>>`-shared), `Client` (per-conn, holds `Framed<TcpStream, LinesCodec>` + `mpsc::UnboundedReceiver`), `Comms(SocketAddr, Tx)`, `Message`. |
| `states.rs` | `ConnStates` enum + `execute(client, registry, world, response)` state-transition function. |
| `cmds.rs` | `dispatch()` — resolves a command via `Registry`, runs the lock check, executes with a `CommandContext`. |
| `prelude.rs` | `Command` trait, `CommandContext`, type aliases. |
| `engine.rs` | Game-logic entry. Pass-through to `cmds::dispatch`; the natural extension point for pre/post-input processing. |
| `hooks.rs` | `Hook` trait + `Hooks` collection. Lifecycle event listeners. |
| `locks.rs` | Lock-expression evaluator (`all()` / `false` / `perm(name)`). |
| `registry.rs` | `Registry` — central index of TypeClasses, Commands, Hooks. Concurrent via `DashMap`. |
| `typeclass.rs` | `TypeClass` trait + 5 built-in types (Character, Room, Item, Exit, Mob). |
| `world.rs` | `WorldApi` — DB facade for command handlers. Wraps `DatabaseHandler` in `Arc<Mutex<>>`. |
| `commands/` | Built-in commands: `look`, `say`, `quit`, `who`. |
| `seed.rs` | `seed_world` — idempotent starting-room seeder. |
| `auth.rs` | argon2 hashing, `AuthBuffer`, name/password validators. |

## Topology

One process. Telnet clients hit `PROXY_ADDR` (default `127.0.0.1:8000`)
directly. WebSocket clients hit `muoxi_web` (default `127.0.0.1:8080`),
which bridges per-client to the same TCP backend. There's no separate
engine listener.

A portal/server split — sockets process and game process talking over a
framed protocol — is on the [roadmap](../../docs/roadmap.md); for now,
one process keeps the surface area smaller.

## State flow

1. `main()` binds `PROXY_ADDR`.
2. Builds `WorldApi` + `Registry`. Registers built-in TypeClasses and
   commands. Seeds the starter room.
3. On accept: builds a `CacheSocket`, persists `(ip, port, uid)` to
   Redis under `Socket:UID:{ip,port,uid}`.
4. Spawns `process()`. UID is recovered from Redis with `gen_uid()` as
   the fallback.
5. `Client::new()` registers the client in `Server.clients` (a shared
   `HashMap<UID, Comms>`).
6. Sends `resources/welcome.txt`. Initial state is `AwaitingName`.
7. Loop: read a line, run `state.execute(client, registry, world,
   response).await`, echo the new state for visibility.
8. When the state reaches `Playing`, each input goes through
   `cmds::dispatch`, which resolves via the Registry, runs the lock
   check, and invokes the matching `Command`.
9. On disconnect or `Quit`, `client_cleanup()` removes the client from
   `Server.clients` and clears its Redis keys.

## State machine

All eight `ConnStates` are wired:

```
       ┌──────────────┐
       │ AwaitingName │◄────── (bad password / no account / lost session)
       └───┬──────────┘
   new │   │ existing
       ▼   ▼
┌──────────┐  ┌────────────┐
│AwNewName │  │AwPassword  │── bad ──► AwaitingName
└────┬─────┘  └─────┬──────┘
     ▼              │ argon2 ok + at_login fires
┌──────────┐        ▼
│AwNewPass │  ┌──────────┐
└────┬─────┘  │ MainMenu │
     ▼        └────┬─────┘
┌──────────┐       │ select N / new <name>
│ConfNewPwd│ ───►  ▼
└────┬─────┘  ┌─────────┐
     │        │ Playing │── quit ──► Quit ── at_disconnect fires
     │        └────┬────┘
     │             │
     └─────────────► (commands via cmds::dispatch + Registry)
```

`Client.auth_buffer` carries `pending_name` and
`first_password_attempt` across transitions. It's cleared on success,
failure, or session loss.

`Client.account_uid` is set on successful auth or new-account creation.
`Client.character_uid` is set on character select or character create.

`at_login` fires on `AwaitingPassword` success — non-cancelable, fanned
out to every registered hook. `at_disconnect` fires from
`client_cleanup` when `client.account_uid` is `Some`; it doesn't fire
on welcome-screen abandons.

Validators (in [`auth.rs`](auth.rs)):

- `is_valid_name`: 3-32 chars, `[A-Za-z][A-Za-z0-9_]*`
- `is_valid_password`: 6+ chars, no whitespace, no null bytes

Password hashing: argon2id with a fresh `OsRng` salt per password, PHC
string stored in `accounts.password_hash`. Verification is constant
time.

`DEV_AUTOLOGIN=1` (read in `main.rs`) skips the state machine and
creates a throwaway `Dev` character in the seeded room — for fast
iteration, not for production.

## Extension surface

Everything pluggable hangs off the `Registry`:

| Method | Purpose |
| --- | --- |
| `register_type(Arc<dyn TypeClass>)` | Add a new in-world type. |
| `register_command(Arc<dyn Command>)` | Add a command. |
| `register_hook(Arc<dyn Hook>)` | Listen for lifecycle events. |

Built-in `TypeClass`es live in [`typeclass.rs`](typeclass.rs): `character`,
`room`, `item`, `exit`, `mob`. Built-in commands live in
[`commands/`](commands/): `look`, `say`, `quit`, `who`.

The lock-expression DSL ([`locks.rs`](locks.rs)) currently supports
`all()`, `false`, and `perm(<name>)`. Anything else denies.

`WorldApi` is the DB surface for command handlers — Diesel calls don't
appear in command code.

## Conventions

- Commands are unit structs deriving `Debug`, with `#[async_trait] impl
  Command`. `name()` and `aliases()` return `&'static str`.
- `Client.lines: Framed<TcpStream, LinesCodec>` is the line-delimited
  protocol. `send()` and `get()` in `lib.rs` are the only public I/O
  helpers.
- Tokio 1.x imports: `tokio_stream::StreamExt` for `.next()` on
  `Framed`; `futures_util::SinkExt` for `.send()`.
- Inside the server modules `crate::*` refers to the library crate
  (`muoxi::*` from outside).

## Things that catch people out

- The engine is a module here, not a separate binary. There's no
  `[[bin]] muoxi_engine`, and no `transfer()` proxy to forward to.
- `tokio::stream::*` doesn't exist in Tokio 1.x. Use
  `tokio_stream::StreamExt`.
- Holding the `Server` mutex across `await` boundaries that touch I/O
  deadlocks.
- Command handlers reach for `WorldApi`, not Diesel directly.
- The dispatcher already resolved your command, so handlers don't need
  to call `Registry::resolve_command` themselves — they receive a
  `CommandContext` with `args` already separated.

## Scheduler

Persistent scheduled jobs live in the `scripts` table. The framework spawns a
single background `Scheduler` task at startup that polls the DB every 50ms
for due jobs and dispatches them to handlers registered via
`Registry::register_script_handler`.

Source:
- [`scheduler.rs`](scheduler.rs) — `Scheduler` task + `ScriptHandler` trait + `ScriptContext`
- [`scripts/`](scripts/) — built-in handlers (`heartbeat`)
- `db::objects::script` — persistence layer (`Script` + `ScriptRepo`)

Built-in handlers: `heartbeat` (logs a tick count; demonstrates the shape). Downstream MUDs add their own:

```rust
use muoxi::scheduler::{ScriptContext, ScriptHandler};
use async_trait::async_trait;
use db::utils::UID;

pub struct MobAi;

#[async_trait]
impl ScriptHandler for MobAi {
    fn key(&self) -> &'static str { "mob_ai" }
    async fn run(
        &self,
        _ctx: &mut ScriptContext<'_>,
        _object_uid: Option<UID>,
        state: serde_json::Value,
    ) -> Result<serde_json::Value, &'static str> {
        // ... mob behavior tick ...
        Ok(state)
    }
}

registry.register_script_handler(Arc::new(MobAi));
```

Schedule a job at runtime:

```rust
let s = registry.world.with_db(|db| {
    db.scripts.create_repeating(
        &mut db.handle,
        Some(mob_uid),         // owning object (None for global)
        "mob_ai",              // handler_key — must match a registered ScriptHandler
        2_000,                 // interval_ms
        &serde_json::json!({}) // initial state
    )
}).await?;
```

Scripts survive restarts. On boot, the scheduler picks up any overdue jobs and runs them once before settling into the steady-state 50ms poll loop. If a handler returns `Err`, the script is disabled (not deleted) so its state remains inspectable.

