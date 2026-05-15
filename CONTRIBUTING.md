# Contributing to MuOxi

Thanks for your interest. There's room for contributions at every level —
framework wiring, API polish, documentation, examples, and entire opt-in
modules.

This file covers the basics. For deeper detail on the codebase, see
[docs/development.md](docs/development.md). For an orientation on
project shape and philosophy, see
[docs/architecture.md](docs/architecture.md).

## Before you start

A few things will save everyone time.

Read the relevant `AGENTS.md`. Each non-trivial subsystem (root,
`db/`, `muoxi/`, `muoxi/src/server/`) has one documenting local
conventions and the things that have caught people out. Skimming it
before changing the subsystem helps.

Open an issue for non-trivial changes. Typo fixes and small docs
improvements can go straight to a PR; touching the state machine, the
hook trait, the lock DSL, or anything else with load-bearing design is
worth a quick conversation first.

Run the tests for the area you changed. The matrix below maps changes
to commands.

## Setup

```bash
git clone https://github.com/duysqubix/MuOxi.git
cd MuOxi
cargo build --workspace
```

`rustup` will fetch the toolchain pinned in
[`rust-toolchain.toml`](rust-toolchain.toml). The default SQLite build
needs no system packages. For the Postgres path, you'll also want
`libpq-dev`.

Sanity check:

```bash
cargo check --workspace
cargo test -p db --features db-sqlite
cargo test -p muoxi --test registry
cargo test -p muoxi --lib auth
```

All should pass on a clean clone.

## Conventions

A handful of conventions hold across the codebase. The full set is in
[docs/development.md](docs/development.md); the gist:

- Edition 2024, stable Rust 1.85.
- Tokio 1.x — individual `AsyncReadExt` / `AsyncWriteExt` imports
  (there's no `tokio::prelude` in 1.x).
- Diesel 2.x — query helpers take `&mut Conn`, macros are namespaced.
- SQLite is the default backend; Postgres is opt-in via
  `--features db-postgres`. The schema stays portable between them.
- Engine and downstream code reach for `WorldApi`, not Diesel directly.
- The `db` crate has `#![deny(missing_docs)]`. Public items get
  docstrings.

When in doubt, match the surrounding code. Genuine inconsistencies are
worth raising in a PR.

## Testing matrix

| You changed | Run |
| --- | --- |
| `db/src/schema.rs` or `migrations/` | `cargo test -p db --features db-sqlite` |
| `db/src/objects/` or `db/src/structures.rs` | same |
| `muoxi/src/server/registry.rs`, `typeclass.rs`, `commands/` | `cargo test -p muoxi --test registry` |
| `muoxi/src/server/auth.rs` | `cargo test -p muoxi --lib auth` |
| Anything in `muoxi/src/server/` (broad) | `cargo check --workspace && cargo clippy --workspace --no-deps` |
| Docker / `Dockerfile` | `docker compose build && docker compose up` then walk through [docs/getting-started.md](docs/getting-started.md) |
| Postgres backend code paths | `cargo check -p db --no-default-features --features db-postgres` |

There's no automated CI yet — local checks are the gate.

## Finding work

The [roadmap](docs/roadmap.md) describes where the project is headed.
If something there interests you, open an issue and we can scope it
together.

Smaller starter tasks if you want to get familiar with the codebase:

- Replace the placeholder `who` (lists every character object in the
  world) with one that lists currently-connected sessions. The
  challenge is threading `Arc<Mutex<Server>>` through `CommandContext`.
- Add `go <direction>` / `<direction>` movement commands. The framework
  doesn't ship them; `docs/world-building.md` has a reference
  implementation.
- Add a Criterion baseline for the hot DB paths. The repo has no perf
  harness yet.

If something interests you that isn't listed, open an issue.

## Commit messages

Conventional-Commits-ish prefixes work well here:

- `feat(scope): ...`
- `fix(scope): ...`
- `refactor(scope): ...`
- `docs(scope): ...`
- `test(scope): ...`
- `chore(scope): ...`

Scope is usually the crate (`db`, `muoxi`) or subsystem (`server`,
`web`, `agents`).

Bodies explain the why. Atomic commits make review easier.

## Pull requests

Keep PRs focused. One wired hook event is a much easier review than
three plus a dispatcher refactor.

Reference issues (`Fixes #N`, `Refs #M`). If your change touches public
API (`WorldApi`, `Command`, `Hook`, `TypeClass`, `Registry`), update
the docstrings and the relevant entry in
[docs/extension-guide.md](docs/extension-guide.md).

Schema changes ship together: the migration, the regenerated
`db/src/schema.rs`, and the structures that exercise the new tables.

## Reporting bugs

Open an issue with what you did, what you expected, what happened, the
commit you're on, and any logs or repro steps. We don't have templates
yet — readable prose is fine.

## Code of conduct

Be kind. We're rebuilding something fun together.

## License

By contributing, you agree your code is released under the same
GPL-3.0 license as the project.
