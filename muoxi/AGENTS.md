# muoxi crate

Application crate with 2 binaries. Edition 2024. Tokio 1.x async runtime. Depends on `db` (path crate) for persistence + caching.

## STRUCTURE

```
muoxi/
├── Cargo.toml          # 2 [[bin]] entries with custom paths; reads workspace deps
└── src/
    ├── server/
    │   ├── main.rs     # muoxi_server bin entrypoint (login + game in one runtime)
    │   ├── cmds.rs     # do_cmd dispatcher + proxy command set
    │   ├── comms.rs    # Server, Client, Comms, Message
    │   ├── engine.rs   # game-logic entry: handle_input (placeholder echo for v0.1)
    │   ├── prelude.rs  # Command trait + CmdSet + cmdset! macro
    │   ├── states.rs   # ConnStates state machine
    │   └── AGENTS.md   # server subsystem documentation
    └── webserver/
        └── webserver.rs # muoxi_web - tokio-tungstenite WS-to-TCP bridge
```

## BINARIES

| Binary | Source | Listens | Status |
|--------|--------|---------|--------|
| `muoxi_server` | `src/server/main.rs` | `127.0.0.1:8000` (override `PROXY_ADDR`) | Combined login + game logic. State machine in `Playing` echoes via `engine::handle_input`. Full auth flow lands in Plan 6. |
| `muoxi_web` | `src/webserver/webserver.rs` | `127.0.0.1:8080` (override `WEB_ADDR`); connects to `PROXY_ADDR` | Dual-purpose on a single port: HTTP GET serves `resources/web/index.html` (browser test client); WS upgrade bridges to TCP backend. Per-client TCP outbound. |

## KEY DEPENDENCIES

- `tokio = "1"` features `["full"]`. Uses `tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncBufReadExt}`, `tokio::net::{TcpListener, TcpStream}`, `tokio::sync::{Mutex, mpsc}`, `tokio::fs::File`.
- `tokio-util = "0.7"` features `["codec"]` for `Framed` / `LinesCodec`.
- `tokio-stream = "0.1"` for `StreamExt::next` on `Framed`.
- `futures-util = "0.3"` for `SinkExt::send` on `Framed`.
- `tokio-tungstenite = "0.24"` - sole websocket implementation; replaced unmaintained `ws` 0.9.
- `async-trait = "0.1"` - required by `Command` trait in `server/prelude.rs`.
- REMOVED in v0.1: `mio`, `mio-extras`, `ws`, `bytes`, `lazy_static`, `hotwatch`.

## CONVENTIONS

- `#![deny(missing_docs)]` on `server/main.rs`. Other modules use `#![allow(missing_docs)]` (`states.rs`, `cmds.rs`).
- Every binary uses `#[tokio::main]` directly.
- Logging: `pretty_env_logger::init()` + ad-hoc `println!`.
- Binary paths under nested dirs use the `path = "src/<bin>/main.rs"` convention so children are looked up as siblings in the same dir.

## ANTI-PATTERNS

- DO NOT add a `muoxi_engine` or `muoxi_staging` binary back. v0.1 is one process.
- DO NOT mix `tokio::prelude` imports - it doesn't exist in Tokio 1.x. Use individual `AsyncReadExt`/`AsyncWriteExt` imports.
- DO NOT change a binary's `[[bin]] path` without also moving its sibling modules - the binary entry file IS the crate root for that target, so child modules must be siblings of it.
- DO NOT put `client.lines.send(msg.into()).await` in code - in tokio_util 0.7 `LinesCodec`'s `Encoder<T: AsRef<str>>` impl is generic, so `.into()` is ambiguous. Use `msg.to_string()` or pass an owned `String` directly.

## RUN

```bash
cargo run --bin muoxi_server                   # then: telnet 127.0.0.1 8000
cargo run --bin muoxi_web                      # ws://127.0.0.1:8080 → tcp 127.0.0.1:8000

WEB_ADDR=127.0.0.1:8888 PROXY_ADDR=127.0.0.1:8000 cargo run --bin muoxi_web
                                               # alt: rebind web port
```

The state machine is incomplete; Plan 6 completes it. See `src/server/AGENTS.md`.
