# server module (the muoxi_server binary)

The unified MuOxi server. Owns the TCP listener, per-client session state, the
connection-state machine, and the in-process game engine. Replaces the old
`staging` + `engine` split.

## FILES

| File | Role |
|------|------|
| `main.rs` | Binary entrypoint. `main()`, `process()` per-client loop, `display_welcome`, `client_cleanup`, `send`/`get` helpers. |
| `comms.rs` | `Server` (`Arc<Mutex<>>`-shared), `Client` (per-conn, holds `Framed<TcpStream, LinesCodec>` + `mpsc::UnboundedReceiver`), `Comms(SocketAddr, Tx)`, `Message` enum. |
| `states.rs` | `ConnStates` enum + `execute(client, response)` state-transition function. |
| `cmds.rs` | `do_cmd` dispatcher + the `proxy_commands` cmdset. |
| `prelude.rs` | `Command` trait, `CmdSet`, `cmdset![]` macro, type aliases. |
| `engine.rs` | Game-logic entry point (`handle_input`). For v0.1 a placeholder; downstream framework users extend it. |

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
2. On accept: build a `CacheSocket`, persist `(ip, port, uid)` to Redis under keys `Socket:UID:{ip,port,uid}`.
3. Spawn `process()`. UID retrieved from Redis with fallback to `gen_uid()` and re-persist.
4. `Client::new()` registers the client in `Server.clients` (shared `HashMap<UID, Comms>`).
5. Send `resources/welcome.txt`. Initial state = `ConnStates::AwaitingName`.
6. Loop: `get(&mut client).await` → `state.execute(client, response).await` → echo new state for visibility.
7. When `state == ConnStates::Playing`, input is forwarded to `engine::handle_input`.
8. On disconnect or `Quit`, `client_cleanup()` removes the client from `Server.clients` and clears Redis keys.

## STATE MACHINE STATUS (CRITICAL)

Plan 6 will complete this. As of post-Plan-2:
- `AwaitingName` is the only state with real logic (other than `Playing`).
- `Playing` routes input through `engine::handle_input` (placeholder echo).
- All other arms fall through to `Quit` (disconnect on first input).

## EXTENSION POINTS (current and planned)

- Add a new command: implement `Command` trait (see `prelude.rs`) and register it in the appropriate state-bound `cmdset![...]` invocation in `states.rs`. Plan 4 makes this a `Registry`.
- Add a new connection state: extend `ConnStates` and add an arm to `execute`.
- Replace game logic: edit `engine::handle_input`. Plan 4 makes this a hook + command dispatch.

## CONVENTIONS

- `cmdset![cmd1, cmd2, ...]` macro is the ONLY blessed way to build a `CmdSet`.
- Commands MUST be unit-structs deriving `Debug, Clone, Hash, Eq, PartialEq`, then `#[async_trait] impl Command`.
- `Client.lines: Framed<TcpStream, LinesCodec>` — line-delimited protocol. `send()` and `get()` (in `main.rs`) are the only public I/O helpers.
- Tokio 1.x imports: `use tokio_stream::StreamExt;` for `.next()` on `Framed`, `use futures_util::SinkExt;` for `.send()`.

## ANTI-PATTERNS

- DO NOT add a `[[bin]] muoxi_engine` back. Engine is a module, not a process.
- DO NOT add a separate `transfer()` proxy. The engine runs in this process; there's no remote to forward to.
- DO NOT use `tokio::stream::*` — Tokio 1.x doesn't have it. Use `tokio_stream::StreamExt`.
- DO NOT hold `Server` mutex across `await` boundaries that touch I/O.
- DO NOT match `client.state` inside `process()` - dispatch IS `state.execute()`.
