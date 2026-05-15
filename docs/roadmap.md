# Roadmap

This doc tracks what MuOxi has, what's coming next, and what's
deliberately out of scope. It's deliberately opinionated — the framework
philosophy (infrastructure, not content) drives the in-scope / out-of-scope
split.

## Shipped in v0.1

The current `master` is a working bare-bones MUD framework. From a fresh
clone, `docker compose up` gives you:

- TCP + WebSocket connectivity
- argon2id auth with full account creation + character select flow
- Persistent state (SQLite default, Postgres opt-in)
- Generic Object / Attribute / Tag model
- `Registry` of TypeClasses + Commands + Hooks
- 5 built-in TypeClasses, 4 built-in commands
- `at_login` / `at_disconnect` hook firing
- Embedded migrations
- `seed_world` for a starting room
- `DEV_AUTOLOGIN` for fast iteration
- Browser test client at `http://localhost:8080`

See the [architecture](architecture.md) doc for the design rationale and
the [extension guide](extension-guide.md) for the full surface.

## Active work

### v0.1.x (bugfixes and small improvements)

These are minor cleanups that don't change the framework's shape.
Reasonable PRs target the current minor version.

- **Hermetic auth-flow integration test**. Currently deferred because
  `CacheSocket::new()` panics if Redis is unreachable. Either mock the
  cache layer in tests or refactor `CacheSocket` to degrade gracefully.
- **`DEV_AUTOLOGIN` character cleanup**. Today, every dev connection creates
  a new `Dev` character object; they accumulate. A startup pass that
  deletes orphaned dev characters would keep the DB tidy.
- **Documented Postgres deployment example**. The path works; we should
  ship a `docker-compose.postgres.yml` overlay.
- **`pre-commit`-style lint checks** in `CONTRIBUTING.md` — recipes for
  running `cargo clippy --workspace --no-deps` and
  `cargo test -p db --features db-sqlite` locally.

### v0.2 (next minor — extension surface completion)

The honest truth from the [extension guide](extension-guide.md): several
trait methods are *declared* but the engine doesn't yet *call* them.
Closing those gaps is the headline goal of v0.2.

- **Wire `Hook::at_object_created`** from `WorldApi::create_object`.
- **Wire `Hook::at_pre_destroy`** with cancel semantics, fired from any
  delete path.
- **Wire `Hook::at_pre_move` + `at_post_move`** from `WorldApi::move_object`.
- **Wire `Hook::at_say`** from the built-in `say` command.
- **`WorldApi::create_object` applies TypeClass defaults** for any
  registered type, not just `"character"`. Today downstream code has to
  do this manually.
- **Server-aware `who`**. Today `who` lists every character object in
  the DB. The fix is to thread `Arc<Mutex<Server>>` (the connected-client
  roster) through `CommandContext`. Without it, "online" isn't a
  first-class concept.
- **Room-broadcast helper** on `WorldApi` (or `CommandContext`), so `say`
  can fan out to other characters in the same room.
- **Cleaner hook-firing closure API**. The current `Hooks::emit` takes
  an `FnMut` that has to re-create `HookContext` per invocation. A
  redesign that passes the context once and lets the trait method
  borrow it would be ergonomically nicer.

### v0.3 (richer auth + locks + builder mode)

- **Lock DSL expansion**: `and`, `or`, `not`, `id(<uid>)`, `holds(<uid>)`,
  arbitrary tag matchers. The current 3-form DSL (`all() / false / perm()`)
  is fine for v0.1 but limits what downstream code can express declaratively.
- **Pluggable auth providers**. Today the framework hardcodes argon2 +
  `accounts` table. Make this a trait so MUDs can swap in OAuth, SAML,
  external LDAP, etc.
- **In-game builder commands**. `@dig <name>`, `@create <type> <name>`,
  `@desc <text>` — gated behind `perm(builder)`. A reference
  implementation should ship as a separate opt-in module.
- **Telnet protocol handling**: IAC byte sequences, MCCP compression,
  GA/EOR prompt markers. Real MUD clients (Mudlet, tt++) expect this.

### v0.4+ (architectural)

- **Portal/server split**. Two processes — proxy (sockets) and engine
  (game logic) — communicating via a framed protocol. Enables hot-reload
  of the engine without disconnecting clients. This was the original
  pre-Plan-2 design; we collapsed it for v0.1 simplicity.
- **Persistent scheduler / scripts**. Evennia-style timed events (heartbeats,
  decay timers, mob AI ticks, respawns, weather rotations). Database-backed
  so jobs survive restarts. Plan 5 in the original v0.1 sketch — punted
  because it isn't required for "people can play."
- **Optional in-process scripting**. PyO3 or mlua for downstream MUDs that
  want to iterate on game logic without recompiling. Strictly opt-in;
  the core engine remains pure Rust.
- **Metrics / observability**. `/metrics` endpoint, structured logs,
  optional OpenTelemetry tracing.

## Out of scope (by design)

These are NOT on any roadmap. The framework's value depends on getting
out of your way for these — every MUD does them differently.

- **A combat system**. Real-time, turn-based, hex-grid, dice-pool, narrative-only
  — these are all valid choices that don't share design vocabulary.
- **A magic / spell / skill system**. Same reasoning.
- **An economy or currency**. From "no economy" to "full Eve-Online-style
  player-driven markets" — pick your poison.
- **A quest engine**. Linear quests, branching quests, faction quests,
  emergent quests — incompatible designs.
- **A specific MUD theme** beyond the placeholder "Limbo" starting room.
- **Default permission roles** beyond the tiny lock DSL. The framework
  ships `perm(NAME)` as a primitive; you decide what `admin`, `gm`,
  `wizard`, `builder`, `player` mean for your MUD.
- **A world-building / OLC system** (in-game room editor). Out of scope
  for the *framework*. Implementing one as an opt-in module is
  reasonable (see v0.3 builder commands) — but the framework won't
  prescribe its shape.
- **Anti-cheat, rate-limiting, abuse mitigation**. Deploy with iptables /
  Cloudflare / fail2ban in front. Application-level rate-limit hooks
  could be a reasonable v0.3+ addition, but the policy is yours.
- **Localization / i18n**. The framework strings are in English. If you
  want a non-English MUD, fork or layer a translation pass on the output
  side.
- **Persistent chat history / logging**. Hook into `at_say` (once wired)
  and write what you want where you want.
- **A built-in web client beyond the dev test page**. The `resources/web/index.html`
  page is a debug aid, not a product. If you want a polished web client,
  build it; reverse-proxy the WS endpoint to your frontend.

## How to influence the roadmap

- **Implementing something on the list**: open an issue describing your
  approach before writing the PR. Saves both sides from wasted work.
- **Wanting something not on the list**: open an issue. If it's a
  framework-vs-downstream judgment call, this is where to make the case.
- **Disagreeing with an "out of scope" item**: same thing. Some of these
  are firmly out of scope (no, MuOxi will never ship a default combat
  system); others might move if the use case is compelling enough.

The roadmap is opinionated, not infallible.

## See also

- [extension-guide.md](extension-guide.md) — the current public extension
  surface, with explicit "declared but not wired" notes
- [architecture.md](architecture.md) — why MuOxi is shaped this way
- [development.md](development.md) — how to hack on the framework itself
