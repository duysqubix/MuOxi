# Topology Collapse Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Merge `muoxi_staging` and `muoxi_engine` into a single `muoxi_server` binary so client commands flow through one Tokio runtime instead of two TCP servers.

**Architecture:** The current `staging_proxy.rs` becomes `server.rs`. Game-logic (currently the trivial echo loop in `muoxi/src/engine/muoxi.rs`) becomes a module `engine` inside the server crate, invoked when a session enters `ConnStates::Playing`. The `transfer()` function and the engine TCP listener disappear. The `webserver` (websocket bridge) keeps its own binary because it's a protocol adapter, not game logic.

**Tech Stack:** Same as parent — Tokio 1.x, no new deps.

---

## File Structure

**Create:**
- `muoxi/src/server.rs` — the new unified binary entrypoint
- `muoxi/src/server/cmds.rs` — moved from `staging/cmds.rs`
- `muoxi/src/server/comms.rs` — moved from `staging/comms.rs`
- `muoxi/src/server/prelude.rs` — moved from `staging/prelude.rs`
- `muoxi/src/server/states.rs` — moved from `staging/states.rs`
- `muoxi/src/server/engine.rs` — game logic; absorbs echo logic from old engine binary
- `muoxi/src/server/AGENTS.md` — replaces the staging-specific AGENTS.md

**Modify:**
- `muoxi/Cargo.toml` — replace `muoxi_engine` and `muoxi_staging` `[[bin]]` entries with one `muoxi_server`
- `muoxi/src/webserver/webserver.rs` — change default `PROXY_ADDR` documentation only (still 127.0.0.1:8000)
- root `AGENTS.md` and `muoxi/AGENTS.md` — reflect the merged binary

**Delete:**
- `muoxi/src/engine/` (entire directory)
- `muoxi/src/staging/` (entire directory — moved to `server/`)

**Test:**
- Existing manual QA suffices: `nc 127.0.0.1 8000 < input.txt` should produce the same login flow output.

---

## Task 1: Add the new directory structure under `muoxi/src/server/`

**Files:**
- Create: `muoxi/src/server/`

- [ ] **Step 1: Create the directory**

```bash
mkdir -p /home/duys/.repos/MuOxi/muoxi/src/server
```

- [ ] **Step 2: Move the four shared modules**

```bash
cd /home/duys/.repos/MuOxi/muoxi/src
git mv staging/cmds.rs server/cmds.rs
git mv staging/comms.rs server/comms.rs
git mv staging/prelude.rs server/prelude.rs
git mv staging/states.rs server/states.rs
```

- [ ] **Step 3: Verify the staging directory now contains only `staging_proxy.rs` and `AGENTS.md`**

Run: `ls /home/duys/.repos/MuOxi/muoxi/src/staging/`
Expected: `staging_proxy.rs  AGENTS.md`

- [ ] **Step 4: Commit**

```bash
git add -A muoxi/src/server muoxi/src/staging
git commit -m "refactor: move staging shared modules under server/"
```

---

## Task 2: Move and rename `staging_proxy.rs` → `server.rs`

**Files:**
- Create: `muoxi/src/server.rs`
- Delete: `muoxi/src/staging/staging_proxy.rs`
- Delete: `muoxi/src/staging/AGENTS.md` (re-created in Task 8)

- [ ] **Step 1: Move the file**

```bash
cd /home/duys/.repos/MuOxi/muoxi/src
git mv staging/staging_proxy.rs server.rs
```

- [ ] **Step 2: Update the module declarations inside `server.rs`**

Open `/home/duys/.repos/MuOxi/muoxi/src/server.rs`. The current top-of-file contains:

```rust
pub mod cmds;
pub mod comms;
pub mod prelude;
pub mod states;
```

Add `pub mod engine;` to the list (engine module is created in Task 4). The block becomes:

```rust
pub mod cmds;
pub mod comms;
pub mod engine;
pub mod prelude;
pub mod states;
```

- [ ] **Step 3: Update the `transfer()` function — remove it (the engine is no longer a remote process)**

Delete the entire `pub async fn transfer(...)` function (lines should be roughly the function block following `client_cleanup`). It's dead code now.

- [ ] **Step 4: Update `main()` — remove the `game_addr` env read since there's no engine to connect to**

In `main()`, delete the line:

```rust
let game_addr: String = env::var("GAME_ADDR").unwrap_or_else(|_| "127.0.0.1:4567".to_string());
```

And the println that mentions `game_addr`:

```rust
println!(
    "TCP Client listening on {} proxying to {}",
    proxy_addr, game_addr
);
```

Replace with:

```rust
println!("MuOxi server listening on {}", proxy_addr);
```

- [ ] **Step 5: Delete the now-stale staging AGENTS.md (will be re-created in Task 8 under server/)**

```bash
git rm muoxi/src/staging/AGENTS.md
```

- [ ] **Step 6: Verify the staging directory is empty and remove it**

```bash
rmdir /home/duys/.repos/MuOxi/muoxi/src/staging
```

If `rmdir` fails because the directory still has tracked files, list and address them.

- [ ] **Step 7: Verify the file structure**

Run: `ls /home/duys/.repos/MuOxi/muoxi/src/`
Expected: `engine/  server/  server.rs  webserver/` (engine/ disappears in Task 3).

- [ ] **Step 8: Commit**

```bash
git add -A muoxi/src/server.rs muoxi/src/staging
git commit -m "refactor: rename staging_proxy.rs to server.rs; remove transfer()"
```

---

## Task 3: Delete the standalone `muoxi_engine` binary

**Files:**
- Delete: `muoxi/src/engine/`

- [ ] **Step 1: Capture the engine's echo logic for reuse in Task 4**

Open `/home/duys/.repos/MuOxi/muoxi/src/engine/muoxi.rs` and note the inner async block that does `socket.read(&mut buf).await` → format as `Game Server > {msg}` → `socket.write_all(...)`. This logic moves into `server/engine.rs` as the placeholder game-logic entry point.

- [ ] **Step 2: Remove the directory**

```bash
cd /home/duys/.repos/MuOxi
git rm -r muoxi/src/engine
```

- [ ] **Step 3: Commit**

```bash
git commit -m "feat: remove standalone muoxi_engine binary"
```

---

## Task 4: Create the `engine` module inside `server/`

**Files:**
- Create: `muoxi/src/server/engine.rs`

- [ ] **Step 1: Write the module**

```rust
//! Game-logic entry point.
//!
//! When a session reaches [`ConnStates::Playing`](crate::states::ConnStates::Playing),
//! the per-line input is dispatched here. For v0.1 this is a placeholder that
//! echoes the input prefixed with `"Game > "`; downstream framework users replace
//! or extend this module.
//!
//! Future direction: this module will host the world API surface that scripts
//! and command handlers call into.

use crate::comms::Client;
use crate::prelude::LinesCodecResult;
use crate::send;

/// Handle a single line of input from a `Playing` client.
///
/// Returns the next state. For now, always returns `Playing`; the auth state
/// machine (Plan 6) gates entry, and `quit` will short-circuit later.
pub async fn handle_input(client: &mut Client, input: &str) -> LinesCodecResult<()> {
    let response = format!("Game > {}", input);
    send(client, &response).await
}
```

- [ ] **Step 2: Verify**

Run: `cd /home/duys/.repos/MuOxi && cargo check -p muoxi`
Expected: errors only from Task 5's pending wiring; this file compiles standalone (it imports from `crate::*` which resolves once `server.rs` declares `pub mod engine;`).

- [ ] **Step 3: Commit**

```bash
git add muoxi/src/server/engine.rs
git commit -m "feat(server): add engine module with handle_input stub"
```

---

## Task 5: Wire `ConnStates::Playing` to call into `engine::handle_input`

**Files:**
- Modify: `muoxi/src/server/states.rs`

- [ ] **Step 1: Add the `Playing` arm to `ConnStates::execute`**

Open `/home/duys/.repos/MuOxi/muoxi/src/server/states.rs`. The current `execute` is:

```rust
pub async fn execute(self, client: &mut Client, response: String) -> LinesCodecResult<Self> {
    match self {
        ConnStates::AwaitingName => { ... }
        _ => Ok(ConnStates::Quit),
    }
}
```

Replace the `_ => Ok(ConnStates::Quit)` arm with:

```rust
ConnStates::Playing => {
    if response.trim().eq_ignore_ascii_case("quit") {
        return Ok(ConnStates::Quit);
    }
    crate::engine::handle_input(client, &response).await?;
    Ok(ConnStates::Playing)
}
_ => Ok(ConnStates::Quit),
```

This is a transitional shim. Plan 6 fills in the other variants properly.

- [ ] **Step 2: Verify**

Run: `cd /home/duys/.repos/MuOxi && cargo check -p muoxi`
Expected: `Finished` with no errors.

- [ ] **Step 3: Commit**

```bash
git add muoxi/src/server/states.rs
git commit -m "feat(server): route Playing state input through engine::handle_input"
```

---

## Task 6: Update `muoxi/Cargo.toml` `[[bin]]` entries

**Files:**
- Modify: `muoxi/Cargo.toml`

- [ ] **Step 1: Replace the engine and staging bins with `muoxi_server`**

Open `/home/duys/.repos/MuOxi/muoxi/Cargo.toml`. Delete:

```toml
[[bin]]
name = "muoxi_engine"
path = "src/engine/muoxi.rs"

[[bin]]
name = "muoxi_staging"
path = "src/staging/staging_proxy.rs"
```

And replace with:

```toml
[[bin]]
name = "muoxi_server"
path = "src/server.rs"
```

The `muoxi_web` entry stays untouched.

- [ ] **Step 2: Verify**

Run: `cd /home/duys/.repos/MuOxi && cargo build --bin muoxi_server`
Expected: `Finished` and a binary at `target/debug/muoxi_server`.

- [ ] **Step 3: Commit**

```bash
git add muoxi/Cargo.toml
git commit -m "build(muoxi): replace engine+staging bins with single muoxi_server"
```

---

## Task 7: Smoke-test the merged binary end-to-end

**Files:** none.

- [ ] **Step 1: Start `muoxi_server` in the background**

```bash
cd /home/duys/.repos/MuOxi
./target/debug/muoxi_server &
SERVER_PID=$!
sleep 1
```

- [ ] **Step 2: Verify the welcome banner**

```bash
echo "" | timeout 3 nc -q1 127.0.0.1 8000 | head -20
```

Expected: the MuOxi ASCII banner from `resources/welcome.txt`. (No call to a separate engine on 4567.)

- [ ] **Step 3: Verify echo from the playing state's placeholder**

For now, the auth state machine still drops to `Quit` on most input — that's Plan 6's scope. Just verify the welcome screen renders and the server responds without crashing.

- [ ] **Step 4: Stop the server**

```bash
kill $SERVER_PID 2>/dev/null
wait $SERVER_PID 2>/dev/null
```

- [ ] **Step 5: Verify the websocket bridge still works against the new server**

```bash
./target/debug/muoxi_server &
SERVER_PID=$!
WEB_ADDR=127.0.0.1:8888 ./target/debug/muoxi_web &
WEB_PID=$!
sleep 2
# (manual: connect a websocket client to ws://127.0.0.1:8888 and verify text round-trips through to the server)
kill $SERVER_PID $WEB_PID 2>/dev/null
```

- [ ] **Step 6: Commit any verification fixups**

```bash
git status
# if empty, this task has nothing to commit; otherwise:
git commit -am "fix: smoke-test fixups for muoxi_server"
```

---

## Task 8: Replace staging AGENTS.md with server AGENTS.md

**Files:**
- Create: `muoxi/src/server/AGENTS.md`

- [ ] **Step 1: Write the new AGENTS.md**

```markdown
# server module (the muoxi_server binary)

The unified MuOxi server. Owns the TCP listener, per-client session state, the
connection-state machine, the command/hook system, and the in-process game
engine. Replaces the old `staging` + `engine` split.

## FILES

| File | Role |
|------|------|
| `../server.rs` | Binary entrypoint. `main()`, `process()` per-client loop, `display_welcome`, `client_cleanup`, `send`/`get` helpers. |
| `comms.rs` | `Server` (`Arc<Mutex<>>`-shared), `Client` (per-conn, holds `Framed<TcpStream, LinesCodec>` + `mpsc::UnboundedReceiver`), `Comms(SocketAddr, Tx)`, `Message` enum. |
| `states.rs` | `ConnStates` enum + `execute(client, response)` state-transition function. |
| `cmds.rs` | `do_cmd` dispatcher + the `proxy_commands` cmdset. |
| `prelude.rs` | `Command` trait, `CmdSet`, `cmdset![]` macro, type aliases. |
| `engine.rs` | Game-logic entry point (`handle_input`). For now a placeholder; downstream framework users extend it. |

## TOPOLOGY

There is exactly ONE process. Telnet clients hit `127.0.0.1:8000` directly.
Websocket clients hit `muoxi_web` (`127.0.0.1:8080`) which bridges to
`127.0.0.1:8000` per-client. There is no separate engine TCP listener.

The portal/server split mentioned in the v0.2 roadmap will reintroduce a
framed protocol between sockets-process and game-process; until then this is
the simplest topology that works.

## STATE FLOW

1. `main()` binds `PROXY_ADDR` (default `127.0.0.1:8000`).
2. On accept: build a `CacheSocket`, persist `(ip, port, uid)` to Redis.
3. Spawn `process()`. Build a `Client`. Send welcome banner.
4. Loop: `get(&mut client).await` → `state.execute(client, response).await` → echo new state.
5. When `state == ConnStates::Playing`, input is forwarded to `engine::handle_input`.
6. On disconnect or `Quit`, `client_cleanup()` removes the client from `Server.clients` and clears Redis keys.

## EXTENSION POINTS (current and planned)

- Add a new command: implement `Command` trait (see `prelude.rs`) and register it in the appropriate state-bound `cmdset![...]` invocation in `states.rs`.
- Add a new connection state: extend `ConnStates` and add an arm to `execute`.
- Replace game logic: edit `engine::handle_input` (Plan 4 makes this a registry).

## ANTI-PATTERNS

- DO NOT add a `[[bin]] muoxi_engine` back. Engine is a module, not a process.
- DO NOT call `transfer()` — it was deleted; there's no remote engine to forward to.
- DO NOT use `tokio::stream::*` — Tokio 1.x doesn't have it. Use `tokio_stream::StreamExt`.
- DO NOT hold `Server` mutex across `await` boundaries that touch I/O.
```

- [ ] **Step 2: Commit**

```bash
git add muoxi/src/server/AGENTS.md
git commit -m "docs(server): document the merged server module"
```

---

## Task 9: Update root and `muoxi/` AGENTS.md

**Files:**
- Modify: `AGENTS.md`
- Modify: `muoxi/AGENTS.md`

- [ ] **Step 1: Root AGENTS.md updates**

In `/home/duys/.repos/MuOxi/AGENTS.md`:

- STRUCTURE section: replace `muoxi/         # 4-binary app crate (engine, staging, watchdog, web)` with `muoxi/         # 2-binary app crate (server, web)`.
- WHERE TO LOOK: change `TCP proxy / connection lifecycle` location to `muoxi/src/server/`. Change `Game logic` location to `muoxi/src/server/engine.rs`. Remove the `JSON → Postgres sync` row entirely.
- COMMANDS: replace the `muoxi_engine`, `muoxi_staging`, `muoxi_watchdog` lines with a single `cargo run --bin muoxi_server  # 127.0.0.1:8000 telnet/internal-tcp surface`.
- ENV VARS: remove `GAME_ADDR`. Keep `PROXY_ADDR`.

- [ ] **Step 2: muoxi/AGENTS.md updates**

In `/home/duys/.repos/MuOxi/muoxi/AGENTS.md`:

- STRUCTURE block: remove the `engine/`, `staging/`, and `watchdog/` lines; add `server.rs` and `server/`.
- BINARIES table: collapse to two rows — `muoxi_server` (port 8000) and `muoxi_web` (port 8080).
- KEY DEPENDENCIES: remove `hotwatch` line.
- ANTI-PATTERNS: keep the existing pattern about not changing `[[bin]]` paths; remove anything specific to the deleted bins.

- [ ] **Step 3: Commit**

```bash
git add AGENTS.md muoxi/AGENTS.md
git commit -m "docs(agents): topology collapsed to muoxi_server + muoxi_web"
```

---

## Verification Summary

A successful run of this plan ends with:

- [ ] `muoxi/src/engine/` and `muoxi/src/staging/` directories no longer exist.
- [ ] `muoxi/src/server.rs` is the binary entrypoint; `muoxi/src/server/` holds `cmds.rs`, `comms.rs`, `engine.rs`, `prelude.rs`, `states.rs`, `AGENTS.md`.
- [ ] `cargo build --workspace` produces `muoxi_server` and `muoxi_web` binaries (and `muoxi_sandbox`, `muoxi_benchmarks` from other crates).
- [ ] `nc 127.0.0.1 8000` against a running `muoxi_server` shows the welcome banner.
- [ ] `muoxi_web` forwards WS traffic to `muoxi_server` end-to-end (re-runs the modernization-era smoke test).
- [ ] No `muoxi_engine`, `muoxi_staging`, or `muoxi_watchdog` references remain in `Cargo.toml` or any AGENTS.md.
