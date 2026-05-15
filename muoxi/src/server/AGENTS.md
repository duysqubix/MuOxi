# server module (the muoxi_server binary)

The unified MuOxi server. Owns the TCP listener, per-client session state, the
connection-state machine, and the registry-driven command dispatch. Now also
the framework's library half — `muoxi/src/lib.rs` re-exports these modules
under `#[path]` attrs so downstream MUDs can depend on `muoxi` as a crate.

## FILES

| File | Role |
|------|------|
| `main.rs` | Binary entrypoint. Constructs `WorldApi` + `Registry`, registers built-ins, spawns `process()` per accepted connection. |
| `comms.rs` | `Server` (`Arc<Mutex<>>`-shared), `Client` (per-conn, holds `Framed<TcpStream, LinesCodec>` + `mpsc::UnboundedReceiver`), `Comms(SocketAddr, Tx)`, `Message` enum. |
| `states.rs` | `ConnStates` enum + `execute(client, registry, world, response)` state-transition function. |
| `cmds.rs` | `dispatch()` — resolves a command via `Registry`, runs the lock check, executes with a `CommandContext`. |
| `prelude.rs` | `Command` trait, `CommandContext`, type aliases. |
| `engine.rs` | Game-logic entry point (`handle_input`). v0.1 pass-through to `cmds::dispatch`; the obvious extension point for pre/post-input processing. |
| `hooks.rs` | `Hook` trait + `Hooks` collection. Lifecycle event listeners (at_login, at_disconnect, at_pre/post_move, at_say, ...). |
| `locks.rs` | Minimal expression evaluator (`all()` / `false` / `perm(name)`). |
| `registry.rs` | `Registry` — central index of TypeClasses, Commands, Hooks. Thread-safe via `DashMap`. |
| `typeclass.rs` | `TypeClass` trait + 5 built-in types (Character, Room, Item, Exit, Mob). |
| `world.rs` | `WorldApi` — DB facade for command handlers. Wraps `DatabaseHandler` in `Arc<Mutex<>>`. |
| `commands/` | Built-in commands: `look`, `say`, `quit`, `who`. |

## TOPOLOGY

There is exactly ONE process. Telnet clients hit `127.0.0.1:8000` (override
with `PROXY_ADDR`) directly. Websocket clients hit `muoxi_web`
(`127.0.0.1:8080`) which bridges per-client to `127.0.0.1:8000`. There is no
separate engine TCP listener.

The portal/server split mentioned in the v0.2 roadmap reintroduces a framed
protocol between sockets-process and game-process; until then this is the
simplest topology that works.

## STATE FLOW

1. `main()` binds `PROXY_ADDR` (default `127.0.0.1:8000`).
2. Builds `WorldApi` + `Registry`, registers built-in TypeClasses + commands.
3. On accept: build a `CacheSocket`, persist `(ip, port, uid)` to Redis under keys `Socket:UID:{ip,port,uid}`.
4. Spawn `process()`. UID retrieved from Redis with fallback to `gen_uid()` and re-persist.
5. `Client::new()` registers the client in `Server.clients` (shared `HashMap<UID, Comms>`).
6. Send `resources/welcome.txt`. Initial state = `ConnStates::AwaitingName`.
7. Loop: `get(&mut client).await` → `state.execute(client, registry, world, response).await` → echo new state for visibility.
8. When `state == ConnStates::Playing`, input is forwarded to `cmds::dispatch` which resolves via Registry, runs the lock check, and invokes the matching `Command`.
9. On disconnect or `Quit`, `client_cleanup()` removes the client from `Server.clients` and clears Redis keys.

## STATE MACHINE

All 8 `ConnStates` have real logic. Flow:

```
       ┌──────────────┐
       │ AwaitingName │◄─────────────── (bad password / no account / lost session)
       └───┬──────────┘
   new │   │ existing-account
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
     └─────────────► (commands run via cmds::dispatch + Registry)
```

* `Client.auth_buffer` carries `pending_name` + `first_password_attempt` across transitions. Cleared on success, failure, or session loss.
* `Client.account_uid` is set on successful auth OR new-account creation.
* `Client.character_uid` is set on character select / character create.
* `at_login` fires on `AwaitingPassword` success (non-cancelable, fan-out to all hooks).
* `at_disconnect` fires from `client_cleanup` when `client.account_uid` is `Some` — never on welcome-screen abandons.

Validators (in [`auth.rs`](file:///home/duys/.repos/MuOxi/muoxi/src/server/auth.rs)):
* `is_valid_name`: 3-32 chars, `[A-Za-z][A-Za-z0-9_]*`
* `is_valid_password`: 6+ chars, no whitespace, no null bytes

Password hashing: argon2id, fresh `OsRng` salt per password, PHC-formatted hash stored in `accounts.password_hash`. Verification is constant-time.

For development with a fast-path Playing flow (skipping the full new-account creation each connect), set `DEV_AUTOLOGIN=1` (see `main.rs`) — this creates a throwaway `Dev` character placed in the seeded room and jumps straight into `ConnStates::Playing`.

## EXTENSION SURFACE

The framework's extension points are all on `Registry`:

| Method | Purpose |
|---|---|
| `register_type(Arc<dyn TypeClass>)` | Add a new in-world type (e.g. "dragon", "vehicle"). |
| `register_command(Arc<dyn Command>)` | Add a new command available everywhere via `resolve_command`. |
| `register_hook(Arc<dyn Hook>)` | Listen to lifecycle events (login, move, say, ...). |

Built-in `TypeClass`es: `character`, `room`, `item`, `exit`, `mob`. See
[`typeclass.rs`](file:///home/duys/.repos/MuOxi/muoxi/src/server/typeclass.rs).

Built-in commands: `look`, `say`, `quit`, `who`. See
[`commands/`](file:///home/duys/.repos/MuOxi/muoxi/src/server/commands/).

Locks: trivial expression DSL — `all()` / `false` / `perm(<name>)`. See
[`locks.rs`](file:///home/duys/.repos/MuOxi/muoxi/src/server/locks.rs).

`WorldApi` is the only DB surface command handlers should touch. Diesel calls
should never appear in command code.

## CONVENTIONS

- Commands MUST be unit-structs deriving `Debug`, then `#[async_trait] impl Command`. `name()` and `aliases()` return `&'static str`.
- `Client.lines: Framed<TcpStream, LinesCodec>` — line-delimited protocol. `send()` and `get()` (in `lib.rs`) are the only public I/O helpers.
- Tokio 1.x imports: `use tokio_stream::StreamExt;` for `.next()` on `Framed`, `use futures_util::SinkExt;` for `.send()`.
- `crate::*` inside server modules refers to the library crate (`muoxi::*` from outside).

## ANTI-PATTERNS

- DO NOT add a `[[bin]] muoxi_engine` back. Engine is a module, not a process.
- DO NOT add a separate `transfer()` proxy. The engine runs in this process; there's no remote to forward to.
- DO NOT use `tokio::stream::*` — Tokio 1.x doesn't have it. Use `tokio_stream::StreamExt`.
- DO NOT hold `Server` mutex across `await` boundaries that touch I/O.
- DO NOT touch Diesel from a command handler. Use `WorldApi` (the facade exists for this reason).
- DO NOT call `Registry::resolve_command` from inside a handler unless absolutely necessary. The dispatcher already did that — handlers receive a `CommandContext` instead.
