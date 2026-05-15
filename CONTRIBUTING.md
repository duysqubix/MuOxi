# Contributing to MuOxi

Thanks for your interest. MuOxi has room for contributions at every
level — framework wiring, downstream-facing API polish, documentation,
examples, and entire opt-in modules.

This file covers the basics for working on the framework itself. For
deeper detail on the codebase, see
[docs/development.md](docs/development.md). For an orientation on the
project shape and philosophy, see
[docs/architecture.md](docs/architecture.md).

## Before you start

Three things:

1. **Read the relevant `AGENTS.md`.** Each non-trivial subsystem
   (root, `db/`, `muoxi/`, `muoxi/src/server/`) has one documenting local
   conventions and anti-patterns. Reading it before you change the
   subsystem saves both sides from rework.
2. **Open an issue for non-trivial changes.** If you're fixing a typo or
   adding a missing docstring, just send the PR. If you're touching the
   state machine, the hook trait, the lock DSL, or anything else with
   load-bearing design — open an issue first.
3. **Run the tests for the area you changed.** See [Testing matrix](#testing-matrix)
   below.

## Setup

```bash
git clone https://github.com/duysqubix/MuOxi.git
cd MuOxi
cargo build --workspace
```

You need Rust 1.85+, but `rustup` will fetch the right toolchain
automatically thanks to [`rust-toolchain.toml`](rust-toolchain.toml). The
default SQLite build needs **zero system packages**.

If you want to work on the Postgres path, also install `libpq-dev` (or
your platform's equivalent).

Sanity check:

```bash
cargo check --workspace
cargo test -p db --features db-sqlite     # 9 tests
cargo test -p muoxi --test registry       # 3 tests
cargo test -p muoxi --lib auth            # 3 tests
```

All should pass on a clean clone.

## Conventions

The non-negotiable ones:

- **Rust edition 2024**, stable channel, MSRV 1.85.
- **Tokio 1.x async runtime**. Use individual `AsyncReadExt` /
  `AsyncWriteExt` imports — `tokio::prelude` doesn't exist in 1.x.
- **Diesel 2.x style**. Every query helper takes `&mut Conn`. Macros
  are namespaced (`diesel::table!`).
- **SQLite is the default backend.** Postgres is opt-in via
  `--features db-postgres`. The compile-error in
  [`db/src/conn.rs`](db/src/conn.rs) enforces that exactly one backend
  is selected.
- **Repos, not raw Diesel.** Engine and downstream code go through
  `WorldApi` and `db.objects.*`, not `diesel::insert_into(...)`.
- **`#![deny(missing_docs)]`** on the `db` crate. Every public item
  needs a docstring.
- **No Postgres-only types in the core schema.** No `BIGINT[]`, no
  `JSONB`, no `LISTEN/NOTIFY`. Keep the schema portable.
- **No type-safety escape hatches.** No `as any`-style suppression of
  real issues. Fix the type, don't silence the compiler.

When in doubt, match the surrounding code's style. If you see a genuine
inconsistency, raising it in a PR is welcome.

For deeper conventions (logging, dependency policy, migration
authoring), see [docs/development.md § Conventions](docs/development.md#conventions).

## Testing matrix

Run the test row(s) that match what you changed:

| You changed | Run |
| --- | --- |
| `db/src/schema.rs` or `migrations/` | `cargo test -p db --features db-sqlite` |
| `db/src/objects/` or `db/src/structures.rs` | same |
| `muoxi/src/server/registry.rs`, `typeclass.rs`, `commands/` | `cargo test -p muoxi --test registry` |
| `muoxi/src/server/auth.rs` | `cargo test -p muoxi --lib auth` |
| Anything in `muoxi/src/server/` (broad) | `cargo check --workspace && cargo clippy --workspace --no-deps` |
| Docker / deploy / `Dockerfile` | `docker compose build && docker compose up` then run [docs/getting-started.md](docs/getting-started.md)'s walkthrough |
| Postgres backend code paths | `cargo check -p db --no-default-features --features db-postgres` |

There's no automated CI configured yet — treat these local checks as
the gate.

## Where to find work

The [roadmap](docs/roadmap.md) is the canonical list. Notably:

- **v0.1.x** — small bugfixes and dev-ergonomics improvements (hermetic
  auth-flow tests, `DEV_AUTOLOGIN` character cleanup, lint recipe).
- **v0.2** — closing the gaps in the extension surface (wiring the
  declared-but-not-fired hooks, generic TypeClass default application,
  server-aware `who`, room-broadcast helper).
- **v0.3** — richer lock DSL, pluggable auth, in-game builder commands.

If something interests you that isn't listed, open an issue. The
roadmap is opinionated, not infallible.

Smaller starter tasks if you want to get familiar with the codebase:

- Replace the placeholder `who` behavior (lists all character objects
  in the world) with one that lists currently-connected sessions. The
  challenge is threading `Arc<Mutex<Server>>` through `CommandContext`.
- Add `go <direction>` / `<direction>` movement commands (the framework
  doesn't ship them; `docs/world-building.md` has a reference
  implementation).
- Add a `cargo bench` baseline using Criterion — there's room for one,
  and there's no perf harness yet (we deleted the 2020-era one).

## Commit messages

Conventional-Commits-ish prefixes:

- `feat(scope): ...` — new functionality
- `fix(scope): ...` — bug fix
- `refactor(scope): ...` — internal change, no behavior shift
- `docs(scope): ...` — documentation-only
- `test(scope): ...` — test additions or changes
- `chore(scope): ...` — tooling, housekeeping

Scope is usually the crate (`db`, `muoxi`) or subsystem (`server`,
`web`, `agents`).

The body should explain the *why* in 1-3 short paragraphs. If a change
is non-obvious, say what alternatives you considered and why this one.

Atomic commits — each commit should be a single logical change that
passes `cargo check --workspace`.

## Pull requests

- Keep PRs focused. A PR that wires `at_pre_move` is much easier to
  review than one that wires three hooks and refactors the dispatcher.
- Reference issues in the PR description (`Fixes #N`, `Refs #M`).
- If your change touches public API (`WorldApi`, `Command`, `Hook`,
  `TypeClass`, `Registry`), add or update the relevant docstrings *and*
  update [docs/extension-guide.md](docs/extension-guide.md) if the
  surface shifts.
- If your change touches the schema, the migration AND the
  `db/src/schema.rs` regeneration go in the same PR (or atomic
  commits).

## Reporting bugs / requesting features

Open an issue. Templates aren't in place yet — for now:

- **Bug**: what you did, what you expected, what happened, logs / repro
  steps, and the commit you're on.
- **Feature**: the use case, what API shape you'd want, and why it
  doesn't fit `WorldApi`/`Command`/`Hook` already.

## Code of conduct

Be respectful. Don't be a jerk. We're rebuilding something fun
together — let's keep it that way.

## License

By contributing, you agree your code is released under the same GPL-3.0
license as the project.
