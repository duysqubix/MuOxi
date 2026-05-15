# Deployment

How to run MuOxi outside of `docker compose up`. For first contact see
[getting-started.md](getting-started.md).

## Environment variables

| Var | Default | Purpose |
| --- | --- | --- |
| `PROXY_ADDR` | `127.0.0.1:8000` | TCP bind address for `muoxi_server`. Use `0.0.0.0:8000` for external connections. |
| `WEB_ADDR` | `127.0.0.1:8080` | Bind address for `muoxi_web` (HTTP + WS). |
| `DATABASE_URL` | `data/world.db` (SQLite) / `postgres://muoxi:muoxi@localhost/muoxi` (PG) | Backend connection string. Path for SQLite, libpq URL for Postgres. |
| `REDIS_SERVER` | `redis://127.0.0.1` | Session cache. The server boots without it but logs errors per connection. |
| `DEV_AUTOLOGIN` | unset | If non-empty and not `0`, new connections skip auth and land in `Playing` as a throwaway `Dev` character. For dev only. |
| `RUST_LOG` | `info,warn,error,test` (forced inside `muoxi_server`) | Standard `env_logger` / `pretty_env_logger` filter. |
| `MUOXI_SERVER_PORT` | `8000` | Docker host-side port remap. |
| `MUOXI_WEB_PORT` | `8080` | Same, for the web bridge. |

## Docker

```bash
docker compose up
```

Builds the multi-stage `Dockerfile`, starts Redis as a sidecar, runs
`muoxi_server` and `muoxi_web`. The SQLite database lives in a named
volume.

```bash
docker compose down -v          # wipe and start fresh
docker compose build            # rebuild after pulling new code
DEV_AUTOLOGIN=1 docker compose up
```

If host ports 8000 or 8080 are taken:

```bash
MUOXI_SERVER_PORT=18000 MUOXI_WEB_PORT=18080 docker compose up
```

The server inside the container still binds 8000; the compose file
maps the host ports.

## Bare metal

```bash
cargo build --workspace --release
target/release/muoxi_server
```

The server reads `resources/welcome.txt` with a CWD-relative path —
run binaries from the repo root, or copy `resources/` alongside the
binary.

The first run creates `data/world.db`. Override with
`DATABASE_URL=/var/lib/muoxi/world.db`.

For external access:

```bash
PROXY_ADDR=0.0.0.0:8000 WEB_ADDR=0.0.0.0:8080 \
  target/release/muoxi_server &
target/release/muoxi_web &
```

## Postgres

Default builds use SQLite (bundled). For Postgres:

```bash
sudo apt install libpq-dev
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

Migrations are embedded into the binary and applied on startup — no
`diesel migration run` invocation needed at runtime. This applies to
both backends.

The schema is intentionally portable. Both backends pass the same
integration tests.

## Redis

The session cache is optional but recommended. Without it, the server
boots and accepts connections; you'll see cache errors per connection
in the log and reconnects can't reuse session UIDs.

```bash
redis-server
REDIS_SERVER=redis://127.0.0.1 target/release/muoxi_server
```

Or a different host:

```bash
REDIS_SERVER=redis://redis.internal:6379 target/release/muoxi_server
```

The docker-compose stack wires this automatically via the `redis`
service hostname.

## SQLite in production

SQLite is the default. It's capable for the kinds of workloads a MUD
generates:

- WAL mode is enabled at connection time, so readers don't block on a
  writer.
- Foreign keys are enabled, so cascading deletes work as expected.
- Writes are serialized. For a single-process server this isn't a
  bottleneck. If you grow into a portal/server split or scale across
  processes, Postgres is the path.

For backup:

```bash
sqlite3 data/world.db ".backup data/world.db.bak"
```

## Logging

`muoxi_server` forces `RUST_LOG=info,warn,error,test` inside `main()`.
Override at compile time or in your fork.

`pretty_env_logger` is the backend; logs go to stderr. The line you're
looking for at boot is:

```
MuOxi server listening on 0.0.0.0:8000
```

If you don't see it, it's almost always a port-binding failure.

## Resource files

The server expects `resources/welcome.txt` at runtime, relative to the
CWD. The web client (`resources/web/index.html`) is embedded into
`muoxi_web` at compile time via `include_str!`, so the file doesn't
need to be present at runtime for the web binary — it's served from
the binary itself.

Replacing either requires a rebuild.

## Reverse proxy

For TLS or a custom domain, terminate at nginx, caddy, or Cloudflare
in front of `muoxi_web`. The bridge speaks plain HTTP + WS on its
bound port; the proxy handles HTTPS + WSS.

```
mud.example.com {
    reverse_proxy localhost:8080
}
```

The bundled browser test client derives its WebSocket URL from
`location.host`, so it works through reverse proxies without
configuration.

Raw TCP/telnet (`muoxi_server` on port 8000) is opened directly. Most
MUD clients don't speak TLS-wrapped telnet anyway.

## See also

- [getting-started.md](getting-started.md) — first contact walkthrough
- [development.md](development.md) — local dev loop
- [roadmap.md](roadmap.md) — where the project is headed
