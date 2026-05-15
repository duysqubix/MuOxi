# Deployment

How to run MuOxi outside of `docker compose up`. For a 5-minute first
contact, see [getting-started.md](getting-started.md).

## Environment variables

| Var | Default | Purpose |
| --- | --- | --- |
| `PROXY_ADDR` | `127.0.0.1:8000` | Where `muoxi_server` binds the TCP listener. Use `0.0.0.0:8000` to accept external connections. |
| `WEB_ADDR` | `127.0.0.1:8080` | Where `muoxi_web` binds (HTTP + WS bridge). |
| `DATABASE_URL` | `data/world.db` (SQLite) / `postgres://muoxi:muoxi@localhost/muoxi` (PG) | Backend connection string. Path-style for SQLite, libpq-style for Postgres. |
| `REDIS_SERVER` | `redis://127.0.0.1` | Session cache. The server boots without Redis but logs errors. |
| `DEV_AUTOLOGIN` | unset | If set to anything non-empty (and not `0`), new connections skip auth and land in `Playing` as a throwaway `Dev` character. **Never set in production.** |
| `RUST_LOG` | `info,warn,error,test` (forced inside `muoxi_server`) | Standard `env_logger` / `pretty_env_logger` filter. |
| `MUOXI_SERVER_PORT` | `8000` | Used by docker-compose to remap the host port. |
| `MUOXI_WEB_PORT` | `8080` | Same, for the web bridge. |

## Docker (default path)

```bash
docker compose up
```

This builds the multi-stage `Dockerfile` (rust:1.85 builder → debian:bookworm-slim
runtime), starts Redis as a sidecar, and runs both `muoxi_server` and
`muoxi_web`. The SQLite database lives in a named volume so it survives
container restarts.

To wipe and start fresh:

```bash
docker compose down -v
```

To rebuild after pulling new code:

```bash
docker compose build
docker compose up
```

To run with autologin for dev:

```bash
DEV_AUTOLOGIN=1 docker compose up
```

### Port collisions

The compose file exposes 8000 and 8080 by default. If those are taken on
your host:

```bash
MUOXI_SERVER_PORT=18000 MUOXI_WEB_PORT=18080 docker compose up
# clients connect to localhost:18000 / localhost:18080
```

The server inside the container still binds 8000 — the compose file maps
the host ports.

## Bare metal

```bash
cargo build --workspace --release
target/release/muoxi_server
```

The server reads `resources/welcome.txt` with a CWD-relative path, so run
binaries from the repo root (or copy `resources/` alongside the binary).

The first run creates `data/world.db` (also CWD-relative). Override with
`DATABASE_URL=/var/lib/muoxi/world.db`.

For external access:

```bash
PROXY_ADDR=0.0.0.0:8000 WEB_ADDR=0.0.0.0:8080 \
  target/release/muoxi_server &
target/release/muoxi_web &
```

## Postgres

Default builds use SQLite (bundled). For a Postgres deployment:

```bash
sudo apt install libpq-dev          # or your platform's libpq package
cargo build --workspace --release --no-default-features --features db-postgres
```

Provision the database:

```bash
sudo -u postgres createuser muoxi
sudo -u postgres createdb -O muoxi muoxi
sudo -u postgres psql -c "ALTER USER muoxi WITH PASSWORD 'muoxi';"
```

Run:

```bash
DATABASE_URL=postgres://muoxi:muoxi@localhost/muoxi \
  target/release/muoxi_server
```

Migrations are embedded into the binary and applied on startup —
**no `diesel migration run` needed**. This applies to both backends.

The schema is intentionally portable (BigInt / Text / Integer; no
`JSONB`, `BIGINT[]`, `LISTEN/NOTIFY`, etc.). Both backends pass the same
integration tests.

## Redis

The session cache is optional but recommended. Without Redis, the server
boots and accepts connections, but you'll see cache errors per connection
in the log and reconnects can't reuse session UIDs.

```bash
redis-server                         # default port 6379
REDIS_SERVER=redis://127.0.0.1 target/release/muoxi_server
```

Or with a different host:

```bash
REDIS_SERVER=redis://redis.internal:6379 target/release/muoxi_server
```

The docker-compose stack handles this automatically via the `redis`
service hostname.

## SQLite — production notes

SQLite is the default backend. It's surprisingly capable for MUDs:

- **Concurrency**: WAL mode is enabled at connection time
  ([`db::conn::configure`](../db/src/conn.rs)), giving you readers
  that don't block on a writer.
- **Single-writer limitation**: SQLite serializes writes. For a v0.1-shape
  MUD (one process, single Tokio runtime), this isn't a bottleneck. If
  you grow to a portal/server split (v0.2 roadmap) or a horizontally
  scaled multi-process deployment, switch to Postgres.
- **Foreign keys**: enabled at connection time. Cascading deletes work
  the way you'd expect.

For backup:

```bash
sqlite3 data/world.db ".backup data/world.db.bak"
```

## Logging

`muoxi_server` forces `RUST_LOG=info,warn,error,test` inside `main()`
(via `unsafe { env::set_var(...) }`). Override at compile time or in your
fork if you want different verbosity.

`pretty_env_logger` is the backend. Logs go to stderr.

Important log lines to look for at boot:

```
DEV_AUTOLOGIN enabled: new connections skip auth and land in room uid=... as 'Dev'.
MuOxi server listening on 0.0.0.0:8000
```

If you don't see `MuOxi server listening`, it's probably a port-binding
failure — check that `PROXY_ADDR` is reachable and not already in use.

## Resource files

The server expects two files at runtime, relative to the CWD:

- `resources/welcome.txt` — login banner sent on connect
- `resources/web/index.html` — **embedded at compile time** into `muoxi_web`
  via `include_str!`, so the file doesn't need to be present at runtime for
  the web binary; it's served from the binary itself

If you replace `resources/welcome.txt`, rebuild and restart. The web
client requires a rebuild because it's embedded.

## Reverse proxy

For TLS or a custom domain, terminate at nginx/caddy/Cloudflare in front
of `muoxi_web`. The bridge speaks plain HTTP + WS on its bound port; the
proxy handles HTTPS + WSS.

Example caddy:

```
mud.example.com {
    reverse_proxy localhost:8080
}
```

The browser test client at `/` works through reverse proxies — it derives
its WebSocket URL from `location.host`, so it just works.

For raw TCP/telnet (`muoxi_server` on port 8000), proxying isn't really a
thing — clients connect directly. Open the port through your firewall;
no TLS at the protocol layer (MUD clients overwhelmingly don't support
TLS-wrapped telnet anyway).

## Production hardening (things you'll want to add)

The framework doesn't ship any of these — they're your responsibility:

- Rate limiting on connection accept (e.g. via iptables / nftables / fail2ban)
- Brute-force protection on auth (e.g. lockout after N failed passwords)
- Persistent logging of in-world chat / commands
- Backup automation for `data/world.db` or your Postgres
- Monitoring (the server doesn't expose a `/metrics` endpoint yet)

These are all reasonable contributions if you want to build them as
opt-in modules.

## Where to next

- [getting-started.md](getting-started.md) — first contact walkthrough
- [development.md](development.md) — local dev loop for hacking on MuOxi
- [roadmap.md](roadmap.md) — what's coming
