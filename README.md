# ![muoxi_logo][logo]

# MuOxi — a MUD framework in Rust

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

MuOxi is a framework for building [online multiplayer text games][wikimudpage]
— MUDs, MUSHes, MUCKs, and their relatives. It handles the parts every
MUD needs (sockets, login, persistence, command dispatch, world state)
and leaves the parts that are *your* MUD to you: combat, magic, plot,
content, feel.

The design owes a lot to [Evennia][evennia], reimagined in Rust.

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

| Surface | URL |
| --- | --- |
| Browser | <http://localhost:8080> |
| Telnet  | `telnet 127.0.0.1 8000` |
| WS CLI  | `wscat -c ws://localhost:8080` |

Create an account, create a character, walk into Limbo. Reconnect later
and your character is still there. The full walkthrough lives in
[docs/getting-started.md](docs/getting-started.md).

If host ports 8000 or 8080 are taken:

```bash
MUOXI_SERVER_PORT=18000 MUOXI_WEB_PORT=18080 docker compose up
```

For fast iteration without typing credentials each restart:

```bash
DEV_AUTOLOGIN=1 docker compose up
```

## Documentation

| If you want to… | Read |
| --- | --- |
| Build a MUD on top of MuOxi | [docs/getting-started.md](docs/getting-started.md) |
| Understand the design | [docs/architecture.md](docs/architecture.md) |
| Know which extension points exist | [docs/extension-guide.md](docs/extension-guide.md) |
| Build out your world | [docs/world-building.md](docs/world-building.md) |
| Deploy MuOxi | [docs/deployment.md](docs/deployment.md) |
| Hack on the framework itself | [docs/development.md](docs/development.md) |
| See where the project is headed | [docs/roadmap.md](docs/roadmap.md) |
| Look up a term | [docs/glossary.md](docs/glossary.md) |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Reach out on [discord][discord]
for design conversations.

## License

GPL-3.0 — see [LICENSE](LICENSE).

[logo]:        .media/cog.png
[wikimudpage]: https://en.wikipedia.org/wiki/MUD
[evennia]:     https://www.evennia.com/
[discord]:     https://discord.gg/H6Sh3CJ
