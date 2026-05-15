# Getting started

This guide walks you from `git clone` to "I have a working MUD running on
my laptop." If you want to understand *why* it's shaped this way, read
[architecture.md](architecture.md) afterwards.

## Prerequisites

You need one of:

- **Docker + Docker Compose** (easiest — recommended path)
- **Rust 1.85+** (`rustup` will fetch the right toolchain automatically
  thanks to [`rust-toolchain.toml`](../rust-toolchain.toml))

No other system packages required. SQLite is bundled into the binary at
compile time; Redis is run as a sidecar container in the docker path.

## Run it (5 minutes)

```bash
git clone https://github.com/duysqubix/MuOxi.git
cd MuOxi
docker compose up
```

This builds the image (≈3 min cold, 30 sec warm), starts Redis and the
unified `muoxi_server` + `muoxi_web` containers, and applies the embedded
migrations to a fresh SQLite database at `data/world.db` inside a named
docker volume.

If host ports 8000 or 8080 are already taken:

```bash
MUOXI_SERVER_PORT=18000 MUOXI_WEB_PORT=18080 docker compose up
```

## Connect

Three options, same backend:

| Surface | Command | Notes |
| --- | --- | --- |
| Browser | open `http://localhost:8080` | Vanilla-JS WebSocket terminal |
| Telnet | `telnet 127.0.0.1 8000` | Or any MUD client (tt++, Mudlet, etc.) |
| WS CLI | `wscat -c ws://localhost:8080` | Useful for scripting |

## Walk through the auth flow

A first-time session looks like this:

```
$ telnet 127.0.0.1 8000

  __  __        ____       _   ______             _
 |  \/  |      / __ \     (_) |  ____|           (_)
 | \  / |_   _| |  | |_  ___  | |__   _ __   __ _ _ _ __   ___
 | |\/| | | | | |  | \ \/ / | |  __| | '_ \ / _` | | '_ \ / _ \
 | |  | | |_| | |__| |>  <| | | |____| | | | (_| | | | | |  __/
 |_|  |_|\__,_|\____//_/\_\_| |______|_| |_|\__, |_|_| |_|\___|
                                             __/ |
                                            |___/

Enter your account name to log in, or type `new` to create one:
> new
Choose an account name (3-32 chars, alphanumeric, start with letter):
> alice
Password (6+ chars, no whitespace):
> hunter2
Confirm password:
> hunter2
Account alice created.
Type `new <name>` to create your first character, or `quit`.

> new Sir_Reginald
Created Sir_Reginald. Entering world.

> look
[Limbo]
You stand in a featureless void. The air feels still and timeless.
A polished stone sits at your feet, and a tired-looking goblin slumps nearby.
Here you see:
  a polished stone
  a tired-looking goblin

> say Hail!
You say, "Hail!"

> who
Characters in the world (1):
  Sir_Reginald

> quit
```

**Disconnect and reconnect** as `alice` with password `hunter2`. Your
character `Sir_Reginald` is still there — accounts and characters persist
in the SQLite database between server restarts.

## Built-in commands

| Command | Aliases | What it does |
| --- | --- | --- |
| `look` | `l` | Describe the current room + visible occupants |
| `say <text>` | `'`, `"` | Speak (currently echoes only to you; broadcast lands later) |
| `who` | — | List characters in the world |
| `quit` | `q`, `exit` | Disconnect |

These are the *only* commands at this stage. Adding more is your job —
see [extension-guide.md](extension-guide.md).

## Skip the auth flow during development

If you're iterating on framework code and don't want to type credentials
every time you restart the server, set `DEV_AUTOLOGIN=1`:

```bash
DEV_AUTOLOGIN=1 docker compose up
```

New connections will skip the entire login state machine and drop you
straight into `Playing` as a throwaway "Dev" character placed in the
seeded starter room. The dev character is recreated on every connection
and accumulates in the DB — don't use this in production.

## Build it without Docker

```bash
cargo build --workspace
cargo run --bin muoxi_server
# in another shell, optionally:
cargo run --bin muoxi_web

# then:
telnet 127.0.0.1 8000
```

The first run creates `data/world.db`. Override the path with
`DATABASE_URL=/path/to/your.db`.

If you want the WebSocket bridge, you also need to start Redis (the
session cache will fail without it but the server keeps running):

```bash
redis-server                  # default port 6379
```

For Postgres instead of SQLite, see [deployment.md](deployment.md#postgres).

## Where to go from here

You've got the framework running. Now decide what kind of MUD you want to
build.

| Goal | Read |
| --- | --- |
| Add a custom command (e.g. `shout`, `inventory`, `attack`) | [extension-guide.md § Commands](extension-guide.md#commands) |
| Define a new in-world type (e.g. `dragon`, `vehicle`, `chest`) | [extension-guide.md § TypeClasses](extension-guide.md#typeclasses) |
| React to login / movement / disconnect | [extension-guide.md § Hooks](extension-guide.md#hooks) |
| Create rooms, items, mobs in the world | [world-building.md](world-building.md) |
| See the worked example in code | [`examples/extension/src/main.rs`](../examples/extension/src/main.rs) |
| Hack on the framework itself | [development.md](development.md) |

## Troubleshooting

**The server starts but I can't connect.**
The default bind is `127.0.0.1:8000` — only accessible from localhost. The
docker compose path overrides this to `0.0.0.0:8000` via the `PROXY_ADDR`
env var.

**The server complains about Redis.**
The server will boot without Redis — cache errors go to stdout but
gameplay continues. To silence them, start a Redis instance and point
`REDIS_SERVER` at it. Docker compose handles this.

**`cargo build` fails with a `libpq` error.**
You've probably switched to the Postgres feature. The default build is
SQLite-only and needs zero system packages. Run:

```bash
cargo build --workspace
```

without any feature flags. If you intentionally want Postgres, install
`libpq-dev`.

**`who` only shows characters that ever existed, not who's connected.**
Correct — `who` currently lists `objects WHERE type_key = 'character'`,
not the in-memory roster of connected sessions. That's a known gap;
proper "online" tracking is on the roadmap.
