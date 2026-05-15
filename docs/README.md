# MuOxi Documentation

MuOxi is a MUD framework in Rust. This directory contains the reference and
guide material for developers who want to build a MUD on top of it, or hack
on the framework itself.

The top-level [`README.md`](../README.md) has the elevator pitch and a 5-minute
quick start. Once you've gotten the docker stack running and connected, come
back here.

## Where to start

| If you want to… | Read |
| --- | --- |
| Build a MUD on top of MuOxi (start here) | [getting-started.md](getting-started.md) |
| Understand the design and why it's shaped this way | [architecture.md](architecture.md) |
| Know which extension points exist and how to register against them | [extension-guide.md](extension-guide.md) |
| Build out your world — rooms, items, mobs, attributes, tags | [world-building.md](world-building.md) |
| Deploy MuOxi (env vars, docker, optional Postgres) | [deployment.md](deployment.md) |
| Hack on MuOxi itself (the framework, not a downstream MUD) | [development.md](development.md) |
| See what's done, what's next, and what's deliberately out of scope | [roadmap.md](roadmap.md) |
| Look up a term | [glossary.md](glossary.md) |

## Repository orientation

If you're reading source rather than docs, the per-subsystem
[`AGENTS.md`](../AGENTS.md) files document local conventions and invariants:

- [Root `AGENTS.md`](../AGENTS.md) — repo-wide code map and conventions
- [`db/AGENTS.md`](../db/AGENTS.md) — persistence layer (Diesel + Redis)
- [`muoxi/AGENTS.md`](../muoxi/AGENTS.md) — application crate (binaries + lib)
- [`muoxi/src/server/AGENTS.md`](../muoxi/src/server/AGENTS.md) — server module internals

## Versioning

Docs in this directory track `master`. If a doc disagrees with the code,
the code is the source of truth — please open a PR or issue.
