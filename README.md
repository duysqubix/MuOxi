
## Recent Updates ##

Due to life.. in general the activity on this project has died down significantly. I have future plans to spin this back up again
but .. well two kids later .. any sort of free time is quite precious. This is still a project I really am interested in doing and most of
the work from here will be design concepts before really any implementation is done. Rust has come a long way since I first started up this project, which may
require to revisit the code.

For the folks who have contributed, I thank you deep from the bottom of my heart, and I urge anyone to continue to help out. I will be working on a roadmap and a list
of features for anyone to take up if they feel like it. My programming skill and project management skills have also significantly changed since I started this. Keep tuned, 
this isn't making progress as much as I'd like, but we'll get there. If someone hasn't done this effort already.


# ![muoxi_logo][logo] 
# MuOxi MUD/MU* Rustic Game Engine v0.1.0
[![Build Status][travisimg]][travislink] [![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0) 


*MuOxi* is a modern library for creating [online multiplayer text
games][wikimudpage] (MU* family) using the powerful features offered by Rust; backed by [Tokio][tokio] and [Diesel][diesel],. 
It allows developers and coders to design and flesh out their worlds in a
fast, safe, and reliable language. Explore MuOxi API the *[rustacean][gh-pages-site]* way Join us on [discord][discord].


## Current Status

The codebase is currently in *alpha* stage. Majority of development is done on the `master` 
branch. There is a working TCP server that allows
for multiple connections and handles them accordingly. Effort is focused at the moment in 
designing the database architecture utilizing [Diesel][diesel] with [PostgreSQL][postgresql] backend.

The dependency stack was modernized in 2026: edition 2024, Tokio 1.x, Diesel 2.x, Redis 0.27, and
`tokio-tungstenite` (replacing the unmaintained `ws` crate). The toolchain is pinned to stable Rust
via [`rust-toolchain.toml`](rust-toolchain.toml). `cargo check --workspace` passes cleanly; building
binaries that link Diesel still requires `libpq-dev` on the host.

## Contributions

Please submit PR's for approval and discussions.
No matter your skill level any sort of effort into this project is extremely welcomed. For those wanting to contribute, checkout the `master` branch
and submit PR's. Any questions or information, we welcome you at our [discord][discord] server. Come on by.

## Road Map

The bare minimum TODO features that must be implemented before I would consider releasing v0.1.1

* Allows for multiple communication protocols (*telnet, MCCP, websocket, etc*)
* Allows for new player creation
* Asks for a name and password
* saves player info (etc. name, password)
* Implements some basic commands: quit, say, tell, shutdown
* ~~Handles players disconnecting or quitting~~
* Implements a periodic message every *n* seconds
* Implements some rudimentary admin control (eg. muting another player)
* Basic cardinal movement
* ~~Implements a backend database, with friendly API tailored to MuOxi~~
* Simple game showcasing features of MuOxi

## Getting Started

MuOxi uses SQLite by default — no external database service required. The default
build needs only a Rust toolchain. The `rust-toolchain.toml` file pins the
project to the matching stable channel.

### Quick start

```bash
git clone https://github.com/duysqubix/MuOxi.git
cd MuOxi
cargo run --bin muoxi_server         # binds 127.0.0.1:8000 (telnet, login + game)
# optional, in another terminal for websocket clients:
cargo run --bin muoxi_web            # binds 127.0.0.1:8080 (bridges to :8000)
```

The first run creates `data/world.db`. Override the path with `DATABASE_URL`.
Migrations under `migrations/` are applied with the `diesel` CLI (optional;
for local hacking the in-memory tests run without it). Connect with any telnet
client:

```bash
telnet 127.0.0.1 8000
```

### Optional: Postgres backend

For larger deployments, opt into the Postgres backend at compile time:

```bash
sudo apt install libpq-dev                    # or your platform's libpq package
cargo build --no-default-features --features db-postgres
DATABASE_URL=postgres://muoxi:muoxi@localhost/muoxi cargo run --bin muoxi_server
```

You'll need to provision the Postgres database yourself (`createdb muoxi`,
`createuser muoxi`, etc.). The same migrations under `migrations/` apply to both
backends.

### Optional: Redis (transient session cache)

MuOxi caches per-connection socket state in Redis. The server still boots
without Redis, but you'll see cache errors in the log:

```bash
redis-server                                  # default port 6379
REDIS_SERVER=redis://127.0.0.1 cargo run --bin muoxi_server
```

### Docker

```bash
docker compose up         # builds image, starts redis + muoxi_server + muoxi_web
telnet 127.0.0.1 8000     # telnet/tt++/mudlet client
open http://127.0.0.1:8080  # browser → built-in JS WebSocket test client
```

If host ports 8000/8080 are taken, override with env vars:

```bash
MUOXI_SERVER_PORT=18000 MUOXI_WEB_PORT=18080 docker compose up
```

The web port serves a tiny HTML page on plain GET requests, and upgrades to a
WebSocket bridge when a client sends `Upgrade: websocket` — both share one
port (8080). The page connects back to `ws://<same-host>` automatically, so it
works through any port override.

## Quick Start Guide

The project ships two binaries:

* **cargo run --bin muoxi_server**
    * Starts the unified MuOxi server on `127.0.0.1:8000` (override `PROXY_ADDR`). One process holds the TCP listener, the login state machine, and the in-process game engine. Direct telnet clients connect on port *8000*.

* **cargo run --bin muoxi_web**
    * Starts the WebSocket bridge listening on `ws://127.0.0.1:8080` (override `WEB_ADDR`).
      Per-WS-client, it opens a fresh outbound TCP connection to the server at
      `127.0.0.1:8000` (override `PROXY_ADDR`). Implemented with `tokio-tungstenite`.

The portal/server split (separate proxy + engine processes with a framed protocol enabling hot-reload) is on the v0.2 roadmap; v0.1 is one process.

## Database Design Architecture

The database design has two layers:

1. **Canonical store** — SQLite (default) or Postgres (opt-in via the `db-postgres` Cargo feature), accessed via [Diesel][diesel].
2. **Transient cache** — [Redis][redis] holds per-session ephemeral state (socket address, UID, throttle counters). Sessions can survive Redis going down; persistent state is never written there.

```
 ┌──────────────┐
 │   Clients    │  telnet / websocket / MCCP
 └──────┬───────┘
        ▼
 ┌──────────────┐
 │ muoxi_server │  proxy + login state machine + game logic (one process)
 └──────┬───────┘
        ├──────────────► Redis  (per-session cache)
        ▼
 ┌──────────────┐
 │   Diesel ORM │  → SQLite (default) | Postgres (opt-in)
 └──────────────┘
```

JSON files used to be canonical in the original design; that has been removed in favor of the database being the single source of truth. JSON now means "import/export payload" only — see `json/README.md`.

## Core Design Architecture

The prototype idea of how the core design is laid out into three seperate objects.
1. Staging/Proxy Server *(Clients will connect to this server and essentially communicate with the engine via this stage)*
2. Game Engine *(all the game logic lies here and reacts to input from connected clients)*
3. Database *(stores information about entities, objects, and game data)* 
4. Communication *( Each supported comm client (MCCP, telnet, websocket) will act as a full-duplex proxy that communicates with the Staging Server)*

The idea is that players will connect via one of the supported communication protocols to the *proxy server*. In this server, clients 
are not actually connected to the game, unless they explicity enter. The *staging area* holds all connected client information such as 
player accounts, different characters for each player, and general settings. When a client acutally connects to the game itself
the server acts as a proxy that relays information from players to the game engine, where the engine will then react to the players input. 
The engine and staging area will be seperated and communicate via a standard TCP server. The reason for this seperation, is to protect players from completely
disconnecting from the game if changes to the game engine is made.

The support for multiple type of connections is a must. Therefore the following shows an example design layout that
has the ability to handle multiple communication protocols. Each comm type will have a unique port that must be addressed
and acts like a proxy to the main Staging Area.

```
------------
| Websocket | <---------------- \
------------                     \
----------                        ---------------------             ---------------
| Telnet | ---------------------->|Proxy/Staging Area | <-- TCP --> | Game Engine |
----------                        ---------------------             ---------------
                                 /
--------                        /
| MCCP | <----------------------
--------
```

This design is still in prototype phase.

## Features and Philosophy

The MuOxi library is aimed at creating a very simplistic and robust library for developers
to experiment and create online text adventure games. 
As it stands the engine has the following capabilities:

* Accepts multiple connections from players
* Maintains a list of connected players
* Hold shared states between connected clients
* Removes clients upon disconnection



## Future/Vision

The concept around MuOxi is not just to recreate an existing MUD game engine in Rust,
but rather to utilize the performance and safety that Rust has to offer. That being said, 
this future vision for MuOxi will change over time, but it needs to fulfill some features
that I think will make this an outstanding project.

1) The Core of MuOxi will be written in Rust, expanding the core will need Rust code
2) The game logic, that handles how Mobs interact, experimental mob AI integration, etc..
   will be handled in Python.
3) *add more here*






[logo]: https://github.com/duysqubix/MuOxi/blob/master/.media/cog.png
[travisimg]: https://travis-ci.org/duysqubix/MuOxi.svg?branch=master
[travislink]: https://travis-ci.org/duysqubix/MuOxi
[wikimudpage]: http://en.wikipedia.org/wiki/MUD
[amethyst]: https://amethyst.rs/
[discord]: https://discord.gg/H6Sh3CJ
[tokio]: https://github.com/tokio-rs/tokio
[diesel]: http://diesel.rs/
[bson]: http://bsonspec.org/
[redis]: https://redis.io/
[gh-pages-site]: https://duysqubix.github.io/MuOxi/
[postgresql]: https://www.postgresql.org/
