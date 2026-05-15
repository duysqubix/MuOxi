# muoxi crate

Application crate with two binaries plus a library half. Edition 2024,
Tokio 1.x async runtime. Depends on the `db` path crate for persistence
and caching.

## Layout

```
muoxi/
├── Cargo.toml          # [lib] + 2 [[bin]] entries; reads workspace deps
└── src/
    ├── lib.rs          # library: process() loop, helpers, SessionConfig;
    │                   # re-exports server modules via #[path] attrs
    ├── server/
    │   ├── main.rs     # muoxi_server bin entrypoint
    │   ├── auth.rs     # argon2 hashing + AuthBuffer + validators
    │   ├── cmds.rs     # dispatch(): resolve → lock → execute
    │   ├── commands/   # built-in commands (look, say, quit, who)
    │   ├── comms.rs    # Server, Client, Comms, Message
    │   ├── engine.rs   # game-logic entry; v0.1 is a pass-through
    │   ├── hooks.rs    # Hook trait + Hooks collection
    │   ├── locks.rs    # lock-expression evaluator
    │   ├── prelude.rs  # Command trait + CommandContext
    │   ├── registry.rs # Registry (types, commands, hooks)
    │   ├── seed.rs     # seed_world (starting room)
    │   ├── states.rs   # ConnStates state machine
    │   ├── typeclass.rs # TypeClass trait + built-ins
    │   ├── world.rs    # WorldApi DB facade
    │   └── AGENTS.md   # server subsystem documentation
    └── webserver/
        └── webserver.rs # muoxi_web — WS-to-TCP bridge + browser test page
```

## Binaries

| Binary | Source | Listens |
| --- | --- | --- |
| `muoxi_server` | `src/server/main.rs` | `127.0.0.1:8000` (override `PROXY_ADDR`) |
| `muoxi_web` | `src/webserver/webserver.rs` | `127.0.0.1:8080` (override `WEB_ADDR`); forwards to `PROXY_ADDR` |

`muoxi_web` is dual-purpose on one port: a plain HTTP GET returns the
browser test client (`resources/web/index.html`, embedded at compile
time); a WebSocket upgrade bridges to the TCP backend.

## Key dependencies

- `tokio = "1"` with feature `full`. Uses
  `tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncBufReadExt}`,
  `tokio::net::{TcpListener, TcpStream}`, `tokio::sync::{Mutex, mpsc}`,
  `tokio::fs::File`.
- `tokio-util = "0.7"` with `codec` for `Framed` / `LinesCodec`.
- `tokio-stream = "0.1"` for `StreamExt::next` on `Framed`.
- `futures-util = "0.3"` for `SinkExt::send` on `Framed`.
- `tokio-tungstenite = "0.24"` for WS.
- `async-trait = "0.1"` for `Command` / `Hook`.
- `dashmap = "6"` for the Registry's concurrent indexes.
- `parking_lot = "0.12"` for the Hooks RwLock.
- `argon2 = "0.5"` for password hashing.

## Conventions

- `#![deny(missing_docs)]` on `server/main.rs`. Other modules use
  `#![allow(missing_docs)]` where the file isn't part of the public lib
  surface (e.g. `states.rs`, `cmds.rs`).
- Each binary uses `#[tokio::main]` directly.
- Logging: `pretty_env_logger::init()` plus ad-hoc `println!` for
  startup banners.
- Library half (`src/lib.rs`) uses `#[path]` attrs to pull modules from
  `src/server/`. This is how external crates (and `examples/extension`)
  reach `muoxi::registry::Registry` etc.

## Things that catch people out

- `tokio::prelude` doesn't exist in Tokio 1.x. Import individual
  `AsyncReadExt` / `AsyncWriteExt` traits instead.
- A binary's `[[bin]] path` and its child modules are coupled. The
  binary entry file is the crate root for that target, so child modules
  must live as siblings of it. Moving the binary path moves the module
  search root.
- `client.lines.send(msg.into()).await` doesn't compile cleanly under
  tokio_util 0.7. `LinesCodec`'s `Encoder<T: AsRef<str>>` impl is
  generic, so `.into()` is ambiguous. Use `msg.to_string()` or pass an
  owned `String` directly.

## Run

```bash
cargo run --bin muoxi_server                   # then: telnet 127.0.0.1 8000
cargo run --bin muoxi_web                      # ws://127.0.0.1:8080
WEB_ADDR=127.0.0.1:8888 cargo run --bin muoxi_web    # alt port
```

For the connection-state machine details, see
[`src/server/AGENTS.md`](src/server/AGENTS.md).
