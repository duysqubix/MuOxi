# staging subsystem

Owns the TCP proxy / login server (`muoxi_staging` binary). Handles login state machine, command dispatch, and proxies bytes to `muoxi_engine` once a client transitions to `Playing` (currently never happens).

## FILES

| File | Role |
|------|------|
| `staging_proxy.rs` | Binary entrypoint. `main()`, `process()` per-client loop, `transfer()` proxy (now via `tokio::io::copy_bidirectional`), `display_welcome()`, `client_cleanup()`, `send`/`get` helpers |
| `comms.rs` | `Server` (shared via `Arc<Mutex<>>`), `Client` (per-connection, holds `Framed<TcpStream, LinesCodec>` + `mpsc::UnboundedReceiver`), `Comms(SocketAddr, Tx)` tuple, `Message` enum |
| `states.rs` | `ConnStates` enum + `execute(client, response) -> Self` state-transition machine |
| `cmds.rs` | `do_cmd()` dispatcher, `proxy_commands` mod (`CmdProxyNew`, `CmdProxyAccount`) |
| `prelude.rs` | `Command` trait (async), `CmdSet` collection, `cmdset![]` macro, `Tx`/`Rx`/`LinesCodecResult`/`CommandResult` aliases |

> Note: `copyover.rs` was removed during the Tokio 1.x port. Its custom `tokio::io::copy` reimplementation is replaced by `tokio::io::copy_bidirectional`.

## CONNECTION LIFECYCLE

1. `main()` binds `PROXY_ADDR` (default `127.0.0.1:8000`).
2. On accept: build a `CacheSocket`, persist `(ip, port, uid)` to Redis under keys `Socket:UID:{ip,port,uid}`.
3. Spawn `process()`. UID retrieved from Redis with fallback to `gen_uid()` and re-persist.
4. `Client::new()` registers the client in `Server.clients` (shared `HashMap<UID, Comms>`).
5. Send `resources/welcome.txt`. Initial state = `ConnStates::AwaitingName`.
6. Loop: `get(&mut client).await` → `state.execute(client, response).await` → echo new state for visibility.
7. On disconnect or `ConnStates::Quit`: `client_cleanup()` removes from `Server.clients` AND `cache.destruct()` removes Redis keys.

## STATE MACHINE STATUS (CRITICAL)

- Only `ConnStates::AwaitingName` is implemented. EVERY other state falls through to `Ok(ConnStates::Quit)` in `states.rs::execute`. Disconnects on first non-name input.
- `CmdProxyNew::execute_cmd` and `CmdProxyAccount::execute_cmd` both return `Ok(())` with empty bodies - login flow is unimplemented.
- `transfer()` proxying staging → engine exists but is NEVER CALLED in the current `process()`. There is no transition from staging to engine yet.

## CONVENTIONS

- `cmdset![cmd1, cmd2, ...]` macro is the ONLY blessed way to build a `CmdSet`. It boxes each as `Box<dyn Command + Send>`.
- Commands MUST be unit-structs deriving `Debug, Clone, Hash, Eq, PartialEq`, then `#[async_trait] impl Command`. Storing object-specific fields is meaningless because dispatch goes through trait objects.
- `Client.lines: Framed<TcpStream, LinesCodec>` — line-delimited protocol. `send()` and `get()` (in `staging_proxy.rs`) are the only public I/O helpers; do NOT call `lines.send/next` directly elsewhere.
- Tokio 1.x imports: `use tokio_stream::StreamExt;` for `.next()` on `Framed`, `use futures_util::SinkExt;` for `.send()`. Internal `Stream` impl on `Client` calls `self.rx.poll_recv(cx)` directly (no longer poll_next - `mpsc::UnboundedReceiver` is not a `Stream` in Tokio 1.x).

## ANTI-PATTERNS

- DO NOT match `client.state` inside `process()` - the dispatch IS `state.execute()`. Adding a parallel match in `process()` is the OLD design and will desync state.
- DO NOT add fields to `dyn Command` impls expecting to read them back - trait-object dispatch hides them.
- DO NOT hold the `Server` mutex across `await` boundaries that touch I/O - lock briefly inside helpers like `Server::broadcast`.
- DO NOT use `tokio::stream::StreamExt` - module is gone. Use `tokio_stream::StreamExt`.
- DO NOT call `Pin::new(&mut self.rx).poll_next(cx)` - `UnboundedReceiver` doesn't impl `Stream` directly. Call `self.rx.poll_recv(cx)` after `self.get_mut()`.

## TODO MARKERS

- `staging_proxy.rs:19`: `// use db::DatabaseHandler;` - intentional, the staging binary is currently DB-free.
- Login flow remains unimplemented; commented out account-creation code from the original was DROPPED during the port.
