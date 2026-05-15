# Getting started

This guide walks from `git clone` to a working MUD running on your
laptop. If you want to understand *why* it's shaped this way,
[architecture.md](architecture.md) is the next read.

## Prerequisites

One of:

- Docker + Docker Compose (the easiest path)
- Rust 1.85+ (`rustup` will fetch the right toolchain via
  [`rust-toolchain.toml`](../rust-toolchain.toml))

No other system packages. SQLite is bundled at compile time; Redis runs
as a sidecar container in the docker path.

## Five-minute path

```bash
git clone https://github.com/duysqubix/MuOxi.git
cd MuOxi
docker compose up
```

This builds the image (~3 min cold, 30s warm), starts Redis, and runs
`muoxi_server` + `muoxi_web`. Migrations apply to a fresh SQLite
database in a named docker volume.

If host ports 8000 or 8080 are taken:

```bash
MUOXI_SERVER_PORT=18000 MUOXI_WEB_PORT=18080 docker compose up
```

## Connect

Three options, same backend:

| Surface | Command |
| --- | --- |
| Browser | <http://localhost:8080> |
| Telnet  | `telnet 127.0.0.1 8000` (or any MUD client) |
| WS CLI  | `wscat -c ws://localhost:8080` |

## Walk through the auth flow

A first-time session:

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

Disconnect, reconnect, and `alice`/`hunter2` brings you back to
`Sir_Reginald`. Accounts and characters persist in SQLite between
restarts.

## Built-in commands

| Command | Aliases | What it does |
| --- | --- | --- |
| `look` | `l` | Describe the current room and visible contents |
| `say <text>` | `'`, `"` | Speak (echoes to you; room broadcast lands later) |
| `who` | — | List characters in the world |
| `quit` | `q`, `exit` | Disconnect |

That's the full command set out of the box. Adding more is what
[extension-guide.md](extension-guide.md) is about.

## Skip auth during development

If you're iterating on framework code and don't want to type
credentials each restart, set `DEV_AUTOLOGIN=1`:

```bash
DEV_AUTOLOGIN=1 docker compose up
```

New connections skip the login state machine entirely and drop you into
`Playing` as a throwaway `Dev` character in the seeded starter room.
Dev characters accumulate in the DB — this is for development, not
production.

## Build without Docker

```bash
cargo build --workspace
cargo run --bin muoxi_server
# in another shell, optionally:
cargo run --bin muoxi_web

# then:
telnet 127.0.0.1 8000
```

The first run creates `data/world.db`. Override with
`DATABASE_URL=/path/to/your.db`. If you want the WebSocket bridge,
start Redis too:

```bash
redis-server
```

For Postgres instead of SQLite, see [deployment.md](deployment.md#postgres).

## Where to go from here

You have the framework running. Time to decide what kind of MUD to
build.

| Goal | Read |
| --- | --- |
| Add a custom command (`shout`, `inventory`, `attack`) | [extension-guide.md](extension-guide.md#commands) |
| Define a new in-world type (`dragon`, `vehicle`, `chest`) | [extension-guide.md](extension-guide.md#type-classes) |
| React to login, movement, disconnect | [extension-guide.md](extension-guide.md#hooks) |
| Create rooms, items, mobs | [world-building.md](world-building.md) |
| See the worked example | [`examples/extension/src/main.rs`](../examples/extension/src/main.rs) |
| Hack on the framework itself | [development.md](development.md) |

## Troubleshooting

**Connection appears to succeed but no banner.** The default bind is
`127.0.0.1:8000` — only reachable from localhost. The docker compose
path sets `PROXY_ADDR=0.0.0.0:8000` automatically.

**Cache errors in the log.** Redis isn't running, or `REDIS_SERVER`
points at something unreachable. The server keeps running; reconnects
just can't reuse session UIDs. Start a local `redis-server` or use the
docker compose stack.

**`cargo build` complains about libpq.** The default build is
SQLite-only and needs nothing extra. If you see libpq errors, you've
switched to the Postgres feature — go back to a default
`cargo build --workspace` or install `libpq-dev`.
