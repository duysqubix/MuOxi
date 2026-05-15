# Extension guide

This is the complete reference for what a downstream Rust developer can
register, override, or replace in MuOxi. Every section follows the same
shape: what it is, the signature, how to register, what the built-ins
do, and a concrete example.

If you're new here, run through [getting-started.md](getting-started.md)
first.

## Status legend

Throughout this doc, surfaces are marked:

- **Wired** — the framework calls or applies this at runtime
- **Declared** — the trait method exists, but the engine doesn't yet emit
  or apply it (planned for v0.2)
- **Internal** — exposed but not part of the public extension contract

Be honest with yourself about which is which when you're planning your MUD.

## The Registry

The central object every extension point is registered against.

**Where**: [`muoxi/src/server/registry.rs`](../muoxi/src/server/registry.rs)

```rust
pub struct Registry {
    types: DashMap<&'static str, Arc<dyn TypeClass>>,
    commands: DashMap<String, Arc<dyn Command>>,
    pub hooks: Hooks,
    pub world: Arc<WorldApi>,
}

impl Registry {
    pub fn new(world: Arc<WorldApi>) -> Self;
    pub fn register_type(&self, t: Arc<dyn TypeClass>);
    pub fn register_command(&self, c: Arc<dyn Command>);
    pub fn register_hook(&self, h: Arc<dyn Hook>);
    pub fn register_builtin_types(&self);
    pub fn get_type(&self, key: &str) -> Option<Arc<dyn TypeClass>>;
    pub fn resolve_command(&self, input: &str) -> Option<Arc<dyn Command>>;
}
```

**Threading**: One `Arc<Registry>` is built in `muoxi_server`'s `main()`,
cloned into every spawned `process()` task, and from there into every
`ConnStates::execute` call and every `Command::execute_cmd` invocation
(via `CommandContext.registry`).

**Embedding pattern** (what a downstream MUD does):

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let world = Arc::new(WorldApi::new(DatabaseHandler::connect()));
    let registry = Arc::new(Registry::new(world.clone()));

    // 1. Framework built-ins
    registry.register_builtin_types();
    muoxi::commands::register_all(&registry);

    // 2. Your additions
    registry.register_type(Arc::new(MyDragonType));
    registry.register_command(Arc::new(CmdShout));
    registry.register_hook(Arc::new(MyAuditLogger));

    // 3. Your seed (or keep the default one)
    seed_world(&world).await?;
    my_world_seed(&world).await?;

    // ... TCP listener + spawn process() per accept, same as muoxi_server's main()
}
```

See [`examples/extension/src/main.rs`](../examples/extension/src/main.rs)
for a compilable version.

## TypeClasses

**Status**: trait fully usable. `default_attributes` + `default_tags` are
**wired** (only for the `"character"` type, via
`WorldApi::create_character`). `default_commands` + `locks` are **declared
but not yet wired** into the runtime.

**Where**: [`muoxi/src/server/typeclass.rs`](../muoxi/src/server/typeclass.rs)

```rust
pub trait TypeClass: Send + Sync {
    fn key(&self) -> &'static str;
    fn description(&self) -> &'static str { "" }
    fn default_attributes(&self) -> HashMap<String, serde_json::Value> { HashMap::new() }
    fn default_tags(&self) -> Vec<(String, String)> { Vec::new() }
    fn default_commands(&self) -> Vec<Arc<dyn Command>> { Vec::new() }    // declared
    fn locks(&self) -> HashMap<&'static str, &'static str> { HashMap::new() }   // declared
}
```

The `key()` is what gets stored in `objects.type_key` and matched against
when you call `world.create_object(key, ...)`.

### Built-in TypeClasses

| key | Defaults | Locks |
| --- | --- | --- |
| `character` | `hp=10`, `desc="A nondescript person."` | `view: all() / examine: perm(builder) / puppet: perm(player)` |
| `room` | `desc="An empty space."` | `view: all() / examine: perm(builder)` |
| `item` | `weight=1`, `desc="An ordinary item."` | `view: all() / examine: all() / use: all()` |
| `exit` | `destination=null` | `traverse: all()` |
| `mob` | `hp=5`, `aggressive=false` | `view: all() / examine: perm(builder)` |

### Register your own

```rust
use async_trait::async_trait;
use muoxi::typeclass::TypeClass;
use std::collections::HashMap;
use std::sync::Arc;

pub struct DragonType;

impl TypeClass for DragonType {
    fn key(&self) -> &'static str { "dragon" }
    fn description(&self) -> &'static str { "A fire-breathing monstrosity." }
    fn default_attributes(&self) -> HashMap<String, serde_json::Value> {
        let mut m = HashMap::new();
        m.insert("hp".into(), serde_json::json!(500));
        m.insert("breath_attack".into(), serde_json::json!("fire"));
        m.insert("treasure_hoard".into(), serde_json::json!(0));
        m
    }
    fn default_tags(&self) -> Vec<(String, String)> {
        vec![("dragon".into(), "kind".into()),
             ("aggressive".into(), "behavior".into())]
    }
}

// At startup:
registry.register_type(Arc::new(DragonType));
```

### Important caveat — generic typeclass auto-apply

`WorldApi::create_object(type_key, name, location)` does **not** automatically
apply the registered TypeClass's defaults. Only `WorldApi::create_character`
does (because it's the only path the framework itself calls today).

Until generic auto-apply lands, you have two options:

**Option A: apply defaults yourself** (most predictable):

```rust
let dragon = world.create_object("dragon", "Vermithrax", Some(room.uid)).await?;
if let Some(tc) = registry.get_type("dragon") {
    for (k, v) in tc.default_attributes() {
        world.set_attribute(dragon.uid, &k, v).await?;
    }
    for (k, cat) in tc.default_tags() {
        world.add_tag(dragon.uid, &k, &cat).await?;
    }
}
```

**Option B: a thin helper in your MUD**:

```rust
pub async fn spawn_typed(
    world: &WorldApi,
    registry: &Registry,
    type_key: &str,
    name: &str,
    location: Option<UID>,
) -> Result<Object, &'static str> {
    let obj = world.create_object(type_key, name, location)
        .await
        .map_err(|_| "create failed")?;
    if let Some(tc) = registry.get_type(type_key) {
        for (k, v) in tc.default_attributes() {
            let _ = world.set_attribute(obj.uid, &k, v).await;
        }
        for (k, cat) in tc.default_tags() {
            let _ = world.add_tag(obj.uid, &k, &cat).await;
        }
    }
    Ok(obj)
}
```

The framework will gain this helper natively in v0.2.

## Commands

**Status**: fully wired. Registry resolution + lock check + execution are
all in production paths.

**Where**: [`muoxi/src/server/prelude.rs`](../muoxi/src/server/prelude.rs)
(trait), [`muoxi/src/server/cmds.rs`](../muoxi/src/server/cmds.rs) (dispatcher).

```rust
pub struct CommandContext<'a> {
    pub client: &'a mut Client,
    pub registry: Arc<Registry>,
    pub world: Arc<WorldApi>,
    pub args: &'a str,
}

#[async_trait]
pub trait Command: Debug + Send + Sync {
    fn name(&self) -> &'static str;
    fn aliases(&self) -> Vec<&'static str> { Vec::new() }
    fn lock(&self) -> &'static str { "all()" }
    async fn execute_cmd(&self, ctx: CommandContext<'_>) -> CommandResult<()>;
}
```

### `CommandContext` fields

| Field | Type | Use it for |
| --- | --- | --- |
| `client` | `&mut Client` | Session state — `client.character_uid`, `client.account_uid`, `client.uid` (session ID), `client.state`, and the `Framed<TcpStream, LinesCodec>` via `client.lines` |
| `registry` | `Arc<Registry>` | Look up other commands, fire hooks manually, find TypeClass defaults |
| `world` | `Arc<WorldApi>` | All DB access (preferred over reaching into `client.uid → DB` directly) |
| `args` | `&str` | Everything after the command name. `"swing axe at goblin"` invoked as `swing` gives `ctx.args == "axe at goblin"` |

### Built-in commands

| Name | Aliases | What it does |
| --- | --- | --- |
| `look` | `l` | Print the current room's name + desc + visible contents |
| `say` | `'`, `"` | Echo "You say, ..." back to the speaker (broadcast not wired yet) |
| `quit` | `q`, `exit` | Print "Goodbye." — the `Playing` arm handles the actual disconnect |
| `who` | — | List `objects WHERE type_key = 'character'` |

### Register your own

```rust
use async_trait::async_trait;
use muoxi::prelude::{Command, CommandContext, CommandResult};
use muoxi::send;

#[derive(Debug)]
pub struct CmdShout;

#[async_trait]
impl Command for CmdShout {
    fn name(&self) -> &'static str { "shout" }
    fn aliases(&self) -> Vec<&'static str> { vec!["yell"] }
    fn lock(&self) -> &'static str { "all()" }

    async fn execute_cmd(&self, ctx: CommandContext<'_>) -> CommandResult<()> {
        if ctx.args.is_empty() {
            let _ = send(ctx.client, "Shout what?").await;
            return Ok(());
        }
        let _ = send(
            ctx.client,
            &format!("You shout, \"{}\"!", ctx.args.to_uppercase()),
        ).await;
        Ok(())
    }
}

// At startup:
registry.register_command(Arc::new(CmdShout));
```

### Locks

A lock is a string returned by `Command::lock()`. Before executing, the
dispatcher calls `locks::check(&world, expr, Some(actor_uid))`.

**Where**: [`muoxi/src/server/locks.rs`](../muoxi/src/server/locks.rs)

Supported forms today:

| Expression | Meaning |
| --- | --- |
| `"all()"` | Always allow (default) |
| `"false"` | Never allow |
| `"perm(NAME)"` | Actor's character must carry the tag `(NAME, "permission")` |
| anything else | Deny conservatively |

To grant a permission to a character:

```rust
world.add_tag(character_uid, "admin", "permission").await?;
```

Then a command with `fn lock(&self) -> &'static str { "perm(admin)" }`
will accept that character.

A richer DSL (`and`, `or`, `not`, `id(<uid>)`, `holds(<uid>)`) is planned —
see [roadmap.md](roadmap.md). Until then, complex permission logic should
go inside `execute_cmd` after a coarse-grained lock gate.

## Hooks

**Status**: Trait fully usable. **Only `at_login` and `at_disconnect` are
currently fired by the engine.** The other five methods are declared
extension points awaiting wiring.

**Where**: [`muoxi/src/server/hooks.rs`](../muoxi/src/server/hooks.rs)

```rust
pub struct HookContext<'a> {
    pub world: &'a WorldApi,
    pub session_uid: Option<UID>,
}

pub type HookResult = Result<(), &'static str>;

#[async_trait]
pub trait Hook: Send + Sync {
    fn name(&self) -> &'static str { "anonymous-hook" }

    // Wired
    async fn at_login(&self, ctx: &mut HookContext<'_>, account_uid: UID) -> HookResult { Ok(()) }
    async fn at_disconnect(&self, ctx: &mut HookContext<'_>, account_uid: UID) -> HookResult { Ok(()) }

    // Declared, not yet wired
    async fn at_object_created(&self, ctx: &mut HookContext<'_>, obj: &Object) -> HookResult { Ok(()) }
    async fn at_pre_destroy(&self, ctx: &mut HookContext<'_>, obj: &Object) -> HookResult { Ok(()) }
    async fn at_pre_move(&self, ctx: &mut HookContext<'_>, obj: &Object, source: Option<UID>, dest: Option<UID>) -> HookResult { Ok(()) }
    async fn at_post_move(&self, ctx: &mut HookContext<'_>, obj: &Object, source: Option<UID>, dest: Option<UID>) -> HookResult { Ok(()) }
    async fn at_say(&self, ctx: &mut HookContext<'_>, speaker: &Object, message: &str) -> HookResult { Ok(()) }
}
```

### Firing mode

`Hooks::emit` is **non-cancelable** — fires all listeners, logs errors at
debug level. `Hooks::emit_cancelable` short-circuits on the first `Err`,
which is how the engine implements the `_pre_*` cancel semantics (once
those are wired).

### Cancelability per method

| Method | Cancelable? | Behavior on `Err` |
| --- | --- | --- |
| `at_login` | No | Logged, ignored |
| `at_disconnect` | No | Logged, ignored |
| `at_object_created` | No (planned) | — |
| `at_pre_destroy` | **Yes** | Object delete is aborted |
| `at_pre_move` | **Yes** | Move is aborted |
| `at_post_move` | No | — |
| `at_say` | **Yes** | Message suppressed |

### Register your own

```rust
use async_trait::async_trait;
use muoxi::hooks::{Hook, HookContext, HookResult};
use db::utils::UID;

pub struct AuditLogger;

#[async_trait]
impl Hook for AuditLogger {
    fn name(&self) -> &'static str { "audit-logger" }

    async fn at_login(&self, _ctx: &mut HookContext<'_>, account_uid: UID) -> HookResult {
        log::info!("login: account_uid={account_uid}");
        Ok(())
    }

    async fn at_disconnect(&self, _ctx: &mut HookContext<'_>, account_uid: UID) -> HookResult {
        log::info!("disconnect: account_uid={account_uid}");
        Ok(())
    }
}

// At startup:
registry.register_hook(Arc::new(AuditLogger));
```

## WorldApi

**Status**: fully wired. This is the DB facade command handlers use; they
should never reach for Diesel directly.

**Where**: [`muoxi/src/server/world.rs`](../muoxi/src/server/world.rs)

### Methods

#### Object lifecycle

| Method | Purpose |
| --- | --- |
| `create_object(type_key, name, location) -> Object` | Insert a new in-world entity (does **not** auto-apply TypeClass defaults) |
| `get_object(uid) -> Option<Object>` | Fetch by UID |
| `move_object(uid, dest) -> usize` | Update `location_uid`; rows affected |
| `contents_of(location) -> Vec<Object>` | Everything where `location_uid == location` |

#### Attributes

| Method | Purpose |
| --- | --- |
| `set_attribute(uid, key, value: serde_json::Value)` | Upsert via SQL ON CONFLICT |
| `get_attribute(uid, key) -> Option<serde_json::Value>` | Parses JSON; `None` if missing |

#### Tags

| Method | Purpose |
| --- | --- |
| `add_tag(uid, key, category)` | Idempotent (ON CONFLICT DO NOTHING) |
| `has_tag(uid, key, category) -> bool` | Used internally by `perm()` locks |
| `objects_with_tag(key, category) -> Vec<UID>` | Cross-object lookup |

#### Accounts

| Method | Purpose |
| --- | --- |
| `find_account_by_name(name) -> Option<Account>` | Case-sensitive lookup |
| `account_password_hash(uid) -> Option<String>` | Just the PHC hash column |
| `create_account(name, password_hash, email) -> Result<Account, &'static str>` | Fails with "name probably taken" on uniqueness violation |

#### Characters

| Method | Purpose |
| --- | --- |
| `list_account_characters(account_uid) -> Vec<Object>` | Ordered by `ordinal` |
| `create_character(registry, account_uid, name, location) -> Result<Object, &'static str>` | Creates `type_key='character'` object, links via `character_accounts`, **applies** the `character` TypeClass's defaults |
| `starting_room() -> Option<UID>` | Finds the room tagged `(starting-room, system)` |

#### Escape hatch

```rust
pub async fn with_db<F, T>(&self, f: F) -> T
where F: FnOnce(&mut DatabaseHandler) -> T
```

Closure receives `&mut DatabaseHandler` (so you get `handle`, `objects`,
`attributes`, `tags`, `character_accounts`, `accounts` simultaneously).
Use when no typed `WorldApi` method covers your query, e.g. complex
multi-table joins. Keep these uses narrow — if you find yourself reaching
for `with_db` repeatedly, that's a signal to add a typed method.

## Lower-level: `db::objects` repos

If you're hacking on the framework itself (not USING it from a downstream
MUD) you may want to bypass `WorldApi` and go straight to the repos.

**Where**: [`db/src/objects/`](../db/src/objects/)

| Repo | Methods |
| --- | --- |
| `ObjectRepo` | `create / get / delete / move_to / rename / list_by_type / contents_of` |
| `AttributeRepo` | `set / get / delete / all` |
| `TagRepo` | `add / remove / has / objects_with / all` |
| `CharacterAccountRepo` | `link / unlink / list_for_account / owner_of` |

These all take `&mut Conn` (Diesel 2.x requirement). They're the source
of truth for `WorldApi`'s typed methods.

**Anti-pattern**: command handlers and hooks should NOT use these directly.
Go through `WorldApi`. The repos are for `db`-crate code, integration
tests, and framework internals.

## SessionConfig + `DEV_AUTOLOGIN`

**Status**: not an extension point — a developer convenience.

**Where**: [`muoxi/src/lib.rs`](../muoxi/src/lib.rs) (the struct);
[`muoxi/src/server/main.rs`](../muoxi/src/server/main.rs) (the env read).

```rust
pub struct SessionConfig {
    pub dev_autologin_room: Option<UID>,
}
```

Passed to `process()` per spawned task. When `dev_autologin_room` is
`Some(uid)`, `process()` skips the auth state machine and creates a
throwaway `Dev` character in that room.

This isn't a general extension API — it's specifically for fast framework
iteration. Don't depend on it for production. It will grow over time
to carry other per-session policies, but each addition will be deliberate.

## `seed_world`

**Status**: a function you replace, not extend.

**Where**: [`muoxi/src/server/seed.rs`](../muoxi/src/server/seed.rs)

```rust
pub async fn seed_world(world: &WorldApi)
    -> Result<UID, Box<dyn Error + Send + Sync>>
```

Idempotent: checks for an existing room tagged `(starting-room, system)`
and skips creation if present. First boot creates:

- Room: `"Limbo"` with a `desc` attribute
- Item: `"a polished stone"` in Limbo
- Mob: `"a tired-looking goblin"` in Limbo

Your downstream MUD's `main()` should call `seed_world(&world).await?`
unconditionally at startup, then call your own seed function for
MUD-specific content:

```rust
let starting_room = seed_world(&world).await?;        // ensures Limbo exists
my_world::seed_kingdom(&world, starting_room).await?; // your content
```

Or, if you want a completely different starting world, write your own
seeder that creates a different room and tags it `(starting-room, system)`,
then skip calling `seed_world`. The `WorldApi::starting_room()` helper
will find yours.

## Surfaces that don't exist yet

For full transparency — if you're planning a non-trivial MUD, you'll
likely want some of these, and you'll need to either contribute them or
work around them:

| Wanted | Status | Workaround |
| --- | --- | --- |
| Per-MUD welcome banner | Hardcoded `resources/welcome.txt` | Replace the file in your fork; container-level mount |
| Cross-room broadcast in `CommandContext` / `WorldApi` | Only internal `Server::broadcast()` exists | Reach into `Server.clients` directly via `with_db` analog (not yet exposed); contribute the API |
| Party / group system | Not present | Build on top of `object_tags` for membership |
| Combat system hooks | Not present | Implement as your own custom `Command`s and `Hook`s |
| Persistent scheduler / NPC AI loop | Planned (roadmap) | Run your own Tokio task that polls `objects WHERE type_key='mob'` |
| Pluggable auth backend | Hardcoded to argon2 + `accounts` table | Fork the state machine for now |
| Custom password policy | Hardcoded in `auth::is_valid_password` | Same |
| Auto-apply TypeClass defaults on `create_object` | Only for `"character"` via `create_character` | Apply manually (see TypeClasses § Important caveat) |
| Auto-emit `at_object_created` / `at_pre_destroy` / `at_pre_move` / `at_post_move` / `at_say` | Only `at_login` / `at_disconnect` fire | Listen for these in your own command implementations; or contribute the wiring |
| Richer lock DSL (`and`, `or`, `not`, `id(...)`, `holds(...)`) | Not present | Use coarse `perm(...)` gating, then check the rest in `execute_cmd` |
| Per-MUD web UI customization | Hardcoded `resources/web/index.html` | Replace the file in your fork; serve a different page from your own HTTP server |
| Server-aware `who` (connected players, not all characters) | `who` lists all `type_key='character'` objects | Implement your own that reads `Server.clients` (requires exposing it through `CommandContext`) |

The [roadmap](roadmap.md) goes into priorities for these items.

## Worked example

The compilable demo lives at
[`examples/extension/src/main.rs`](../examples/extension/src/main.rs).
It registers a custom command (`shout`) and a custom TypeClass (`dragon`),
constructs a Registry, and verifies `resolve_command` finds the new
command. It doesn't bind a listener — wire that yourself when you're
building a real MUD by copying `muoxi/src/server/main.rs`.

```bash
cargo run --bin muoxi-example-extension
```
