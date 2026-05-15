# muoxi crate

Multi-binary application crate. Edition 2024. Tokio 1.x async runtime. Depends on `db` (path crate) for persistence + caching.

## STRUCTURE

```
muoxi/
├── Cargo.toml          # 4 [[bin]] entries with custom paths; reads workspace deps
└── src/
    ├── engine/
    │   └── muoxi.rs    # muoxi_engine - echo TCP server (Tokio 1.x AsyncReadExt)
    ├── staging/        # muoxi_staging - main proxy/login (see staging/AGENTS.md)
    ├── watchdog/
    │   └── watchdog.rs # muoxi_watchdog - hotwatch 0.5 JSON → Postgres syncer
    └── webserver/
        └── webserver.rs # muoxi_web - tokio-tungstenite WS-to-TCP bridge
```

## BINARIES

| Binary | Source | Listens | Status |
|--------|--------|---------|--------|
| `muoxi_engine` | `src/engine/muoxi.rs` | `127.0.0.1:4567` (override `GAME_ADDR`) | Echo only - placeholder for real game logic. Verified end-to-end. |
| `muoxi_staging` | `src/staging/staging_proxy.rs` | `127.0.0.1:8000` (override `PROXY_ADDR`) | Working proxy. Login state machine still incomplete. |
| `muoxi_watchdog` | `src/watchdog/watchdog.rs` | n/a (file watcher) | Watches `json/accounts.json` + `json/characters.json` |
| `muoxi_web` | `src/webserver/webserver.rs` | `127.0.0.1:8080` (override `WEB_ADDR`); connects to `PROXY_ADDR` | Working WS-to-TCP bridge. Per-client TCP outbound. |

## KEY DEPENDENCIES

- `tokio = "1"` features `["full"]`. Uses `tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncBufReadExt}`, `tokio::net::{TcpListener, TcpStream}`, `tokio::sync::{Mutex, mpsc}`, `tokio::fs::File`.
- `tokio-util = "0.7"` features `["codec"]` for `Framed` / `LinesCodec`.
- `tokio-stream = "0.1"` for `StreamExt::next` on `Framed`.
- `futures-util = "0.3"` for `SinkExt::send` on `Framed`.
- `tokio-tungstenite = "0.24"` - sole websocket implementation; replaced unmaintained `ws` 0.9.
- `hotwatch = "0.5"` - filesystem watcher (notify-based; Event API uses `EventKind::Modify(_)`).
- `async-trait = "0.1"` - required by `Command` trait in `staging/prelude.rs`.
- REMOVED: `mio`, `mio-extras`, `ws`, `bytes`, `lazy_static` (the watchdog now uses `std::sync::OnceLock`).

## CONVENTIONS

- `#![deny(missing_docs)]` on `staging_proxy.rs` and `watchdog.rs`. Other modules use `#![allow(missing_docs)]` (`states.rs`, `cmds.rs`).
- Every binary uses `#[tokio::main]` directly (the `ws` crate's sync `listen()` is gone, so `webserver.rs` is now single-runtime like the others).
- Logging: `pretty_env_logger::init()` + ad-hoc `println!`.

## ANTI-PATTERNS

- DO NOT mix `tokio::prelude` imports - it doesn't exist in Tokio 1.x. Use individual `AsyncReadExt`/`AsyncWriteExt` imports.
- DO NOT change a binary's `[[bin]] path` without also moving its `mod.rs` declarations - `staging_proxy.rs` declares `pub mod cmds; pub mod comms; ...` so the binary path determines module resolution.
- DO NOT call DB code from `engine/muoxi.rs` yet - it is intentionally a pure echo server until the engine design is finalized. The binary is also the ONLY way to verify changes without `libpq` installed.
- DO NOT put `client.lines.send(msg.into()).await` in code - in tokio_util 0.7 `LinesCodec`'s `Encoder<T: AsRef<str>>` impl is generic, so `.into()` is ambiguous. Use `msg.to_string()` or pass an owned `String` directly.

## RUN

```bash
cargo run --bin muoxi_staging                  # then: telnet 127.0.0.1 8000
cargo run --bin muoxi_watchdog                 # in another terminal (needs libpq + Postgres)
cargo run --bin muoxi_engine                   # in another terminal (port 4567)
cargo run --bin muoxi_web                      # ws://127.0.0.1:8080 → tcp 127.0.0.1:8000

WEB_ADDR=127.0.0.1:8888 PROXY_ADDR=127.0.0.1:4567 cargo run --bin muoxi_web
                                               # alt: forward directly to engine for testing
```

`muoxi_staging` is designed to PROXY into `muoxi_engine` once a client transitions to `ConnStates::Playing`. The state machine is currently incomplete - see `src/staging/AGENTS.md`.
