# ![muoxi_logo][logo]

# MuOxi — a MUD framework in Rust

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

MuOxi is a framework for building [online multiplayer text games][wikimudpage]
(MUDs, MUSHes, MUCKs — the MU\* family). It ships the parts every MUD
needs — sockets, login, persistence, command dispatch, world state — and
gets out of the way for the parts that are *your* MUD: combat, magic,
crafting, economy, plot.

The closest spiritual ancestor is [Evennia][evennia] (Python). MuOxi
borrows the design — generic typed objects, freeform attributes, hook-based
extension — and brings Rust's type system, async runtime, and persistence
story along.

```
                                          ┌────────────┐
   tt++ / telnet      ─tcp→  ┌───────────┐│   redis    │
   browser / WS       ─http→ │muoxi_server│└────┬───────┘
   muoxi_web bridge   ─ws──→ └─────┬─────┘     │
                                   │           ▼
                                   ▼     ┌────────────┐
                            ┌──────────┐ │  Diesel    │
                            │ Registry │ │  SQLite    │
                            └──────────┘ │  (default) │
                              types,     │   or       │
                              cmds,      │  Postgres  │
                              hooks      └────────────┘
```

## Quick start

```bash
git clone https://github.com/duysqubix/MuOxi.git
cd MuOxi
docker compose up
```

Then connect:

| Surface | URL | Notes |
| --- | --- | --- |
| Browser | <http://localhost:8080> | In-browser WebSocket terminal |
| Telnet | `telnet 127.0.0.1 8000` | Or any MUD client |
| WS CLI | `wscat -c ws://localhost:8080` | Useful for scripting |

Create an account, create a character, walk into "Limbo." Disconnect,
reconnect, and your character is still there. The full walkthrough is in
[docs/getting-started.md](docs/getting-started.md).

If host ports 8000 / 8080 are taken:

```bash
MUOXI_SERVER_PORT=18000 MUOXI_WEB_PORT=18080 docker compose up
```

For fast framework iteration without typing credentials each restart:

```bash
DEV_AUTOLOGIN=1 docker compose up
```

## What MuOxi provides

- TCP + WebSocket connectivity with a built-in browser test client
- argon2id authentication with full account / character creation flow
- Persistent state via Diesel ORM — SQLite by default (zero system
  deps), Postgres opt-in
- Generic Object / Attribute / Tag model — add new in-world types
  without schema migrations
- A `Registry` of `TypeClass`es, commands, and hooks — the extension
  surface downstream MUDs register against
- 5 built-in TypeClasses (Character, Room, Item, Exit, Mob), 4 built-in
  commands (`look`, `say`, `quit`, `who`)
- `at_login` / `at_disconnect` lifecycle hooks
- Embedded migrations — fresh installs work out of the box
- Toolchain pinned to stable Rust 1.85

## What MuOxi does NOT provide

These are deliberately out of scope — they belong in *your* MUD, not the
framework:

- A combat system • A magic / spell / skill system • An economy or
  currency • A quest engine • A specific theme or content beyond the
  placeholder starting room • Default permission roles beyond the tiny
  lock DSL • A world-building / OLC system • Localization

See [docs/roadmap.md](docs/roadmap.md) for the full scope discussion.

## Documentation

| If you want to… | Read |
| --- | --- |
| Build a MUD on top of MuOxi | [docs/getting-started.md](docs/getting-started.md) |
| Understand the design | [docs/architecture.md](docs/architecture.md) |
| Know which extension points exist | [docs/extension-guide.md](docs/extension-guide.md) |
| Build out your world | [docs/world-building.md](docs/world-building.md) |
| Deploy MuOxi | [docs/deployment.md](docs/deployment.md) |
| Hack on the framework itself | [docs/development.md](docs/development.md) |
| See the roadmap | [docs/roadmap.md](docs/roadmap.md) |
| Look up a term | [docs/glossary.md](docs/glossary.md) |

## Project status

v0.1 is shipped — a working bare-bones MUD framework. The
[roadmap](docs/roadmap.md) tracks v0.2 (closing extension-surface gaps)
and beyond.

## Contributing

PRs welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, conventions,
and the testing matrix. Reach out on [discord][discord] for design
conversations.

## License

GPL-3.0 — see [LICENSE](LICENSE).

[logo]:        .media/cog.png
[wikimudpage]: https://en.wikipedia.org/wiki/MUD
[evennia]:     https://www.evennia.com/
[discord]:     https://discord.gg/H6Sh3CJ
