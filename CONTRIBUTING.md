# Contributing to MuOxi

Thanks for your interest. MuOxi is in active revival and there's plenty to do
at every skill level. This doc covers setup, conventions, and where to look for
work.

## Quick orientation

- **What this project is**: a Rust-based MUD/MU\* framework, Evennia-inspired.
  See the [README](README.md) for the elevator pitch and architecture.
- **What state it's in**: v0.1 in progress. Six axis-plans under
  [`docs/superpowers/plans/`](docs/superpowers/plans/) — Plans 1, 2, 3 are
  done; Plan 4 is next.
- **Where to start**: pick a plan that's not yet started (4, 5, or 6), or
  scope a smaller task from the open issues.

## Setup

```bash
git clone https://github.com/duysqubix/MuOxi.git
cd MuOxi

# Sanity check — should pass without any system packages
cargo build --workspace
cargo test -p db --features db-sqlite

# Optional: bring up the full stack
docker compose up
```

You need Rust 1.85+ (pinned via `rust-toolchain.toml`, so `rustup` will fetch
the right one automatically). No other system dependencies for the default
SQLite build.

If you want to work on the Postgres path, you'll also need `libpq-dev` (or
your platform's equivalent) and a local Postgres instance.

## Workflow

1. Fork → branch → PR is the norm.
2. **Atomic commits** are strongly preferred. Each commit should be a single
   logical change that passes `cargo check --workspace` and (where relevant)
   `cargo test`. The existing history is a reasonable model.
3. Run `cargo check --workspace` and `cargo clippy --workspace --no-deps`
   before pushing.
4. If you're touching `db/`, run `cargo test -p db --features db-sqlite`.
5. Reference the relevant `AGENTS.md` for the subsystem you're editing — they
   capture local conventions and anti-patterns that aren't obvious from the
   code alone.

## Conventions

These are non-negotiable for the codebase (they're enforced by the build, by
the lints, or by code review):

- **Rust edition 2024**, stable channel, MSRV 1.85.
- **Tokio 1.x async runtime**. Use individual `AsyncReadExt` / `AsyncWriteExt`
  imports — `tokio::prelude` doesn't exist in 1.x.
- **Diesel 2.x style**. Every query helper takes `&mut Conn` (Diesel 2.x
  requirement). Macros are namespaced (`diesel::table!`).
- **SQLite is the default backend.** Postgres is opt-in via
  `--features db-postgres`. The compile-error in `db/src/conn.rs` enforces
  that exactly one backend is selected.
- **Repos, not raw Diesel.** Engine and downstream code go through
  `db.objects.create(...)` etc. — not `diesel::insert_into(...)`.
- **`#![deny(missing_docs)]`** on the `db` crate and `muoxi/src/server/main.rs`.
  Every public item needs a docstring.
- **No `BIGINT[]`, `JSONB`, `LISTEN/NOTIFY`, or other Postgres-only types**
  in the core schema. Keep it portable so SQLite stays viable.
- **No `as any`-style escape hatches.** Type errors get fixed, not silenced.
- **JSON canonical / watchdog are dead.** The database is the single source of
  truth. The `json/` directory is for import/export payloads only.

When in doubt, match the surrounding code's style and patterns. If you spot
a genuine inconsistency, raising it in a PR is welcome.

## Where to find work

### Reading the plans

The six v0.1 axis-plans live under
[`docs/superpowers/plans/`](docs/superpowers/plans/). Each plan is broken into
self-contained tasks with code blocks, verification commands, and per-task
commit messages. They're written so an outside contributor (or AI agent) can
execute them step-by-step.

- **Plan 4** ([command-hook-registry](docs/superpowers/plans/2026-05-07-command-hook-registry.md))
  — TypeClass / Hook / Registry. 14 tasks. The biggest user-facing milestone.
- **Plan 5** ([scheduler](docs/superpowers/plans/2026-05-07-scheduler.md))
  — persistent timed scripts. 9 tasks.
- **Plan 6** ([auth-state-machine](docs/superpowers/plans/2026-05-07-auth-state-machine.md))
  — complete login flow with argon2. 12 tasks.

If you want to tackle a whole plan, the
[MASTER-PLAN](docs/superpowers/plans/2026-05-07-MASTER-PLAN.md) has a "resume
point" header that orients you on current state.

### Smaller starter tasks

Things that don't fit a plan but would help:

- `benchmarks/db_100_000.json` fixture generator — the benchmark binary
  references it but it's not committed (and shouldn't be — generate-on-demand
  is the right call).
- More integration test coverage in `db/tests/`, especially around the
  cross-entity queries (`tags::objects_with`, `objects::contents_of`).
- The `muoxi_sandbox` binary is currently a 40-line Redis round-trip. Could
  grow into a useful manual-test surface for the object model.

If something interests you that isn't listed, open an issue or come chat on
[discord](https://discord.gg/H6Sh3CJ) first so we don't double-up work.

## Commit messages

The recent history uses Conventional-Commits-ish prefixes:

- `feat(scope): ...` for new functionality
- `fix(scope): ...` for bug fixes
- `refactor(scope): ...` for internal changes that don't change behaviour
- `docs(scope): ...` for documentation-only changes
- `test(scope): ...` for test additions / changes
- `chore(scope): ...` for tooling / housekeeping

Scope is usually the crate (`db`, `muoxi`) or subsystem (`server`, `web`,
`plans`, `agents`).

Body should explain the *why* in 1-3 short paragraphs. Reference plan tasks
where applicable (`Plan 4 Task 7`).

## Pull requests

- Keep PRs focused. A PR that does Plan 4 Task 1 + 2 + 3 + 4 is much easier to
  review than one that does Plans 4 and 5 simultaneously.
- The CI is currently minimal (Travis legacy file only, no active workflow).
  Run the local sanity checks above before pushing.
- If your change touches the schema, include the new migration AND the
  corresponding `db/src/schema.rs` update AND a `down.sql` that actually
  reverts.
- For PRs that add public API on the `db` crate, include a docstring per the
  `#![deny(missing_docs)]` rule and (ideally) an integration test under
  `db/tests/`.

## Code of conduct

Be respectful. Don't be a jerk. We're rebuilding something fun together —
let's keep it that way.

## License

By contributing, you agree your code is released under the same GPL-3.0
license as the project.
