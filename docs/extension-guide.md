# Extension guide

This is the reference for what a downstream Rust developer can register
or override in MuOxi. Read [getting-started.md](getting-started.md)
first if you haven't run through it.

The framework's extension points all live on the `Registry`: type classes,
commands, and hooks. Everything else (the world facade, the lock evaluator,
the seed function) is reachable from your `main()` and from the
`CommandContext` your handlers receive.

## The Registry

The central object every extension hangs off.

```rust
pub struct Registry {
    pub hooks: Hooks,
    pub world: Arc<WorldApi>,
    // (internal: types, commands DashMaps)
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

One `Arc<Registry>` is built in `muoxi_server`'s `main()` and threaded
into every spawned `process()` task, into every `ConnStates::execute`
call, and from there into every `Command::execute_cmd` invocation
via `CommandContext.registry`.

A downstream MUD's `main()` looks like the framework's
`muoxi_server/main.rs` with your registrations inserted:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let world = Arc::new(WorldApi::new(DatabaseHandler::connect()));
    let registry = Arc::new(Registry::new(world.clone()));

    registry.register_builtin_types();
    muoxi::commands::register_all(&registry);

    registry.register_type(Arc::new(MyDragonType));
    registry.register_command(Arc::new(CmdShout));
    registry.register_hook(Arc::new(MyAuditLogger));

    seed_world(&world).await?;
    my_world_seed(&world).await?;

    // bind, accept, spawn process() — same as muoxi_server/main.rs
}
```

[`examples/extension/src/main.rs`](../examples/extension/src/main.rs)
is a compilable version.

## Type classes

A `TypeClass` defines an in-world entity type — its key, defaults, and
locks.

Source:
[`muoxi/src/server/typeclass.rs`](../muoxi/src/server/typeclass.rs).

```rust
pub trait TypeClass: Send + Sync {
    fn key(&self) -> &'static str;
    fn description(&self) -> &'static str { "" }
    fn default_attributes(&self) -> HashMap<String, serde_json::Value> { HashMap::new() }
    fn default_tags(&self) -> Vec<(String, String)> { Vec::new() }
    fn default_commands(&self) -> Vec<Arc<dyn Command>> { Vec::new() }
    fn locks(&self) -> HashMap<&'static str, &'static str> { HashMap::new() }
}
```

`key()` is what gets stored in `objects.type_key` and matched against
when you call `world.create_object(key, ...)`.

The framework ships five built-in types: `character`, `room`, `item`,
`exit`, and `mob`. Each has reasonable default attributes
(`character` has `hp=10` and a `desc`; `item` has `weight=1`; etc.).

To define a new type:

```rust
use muoxi::typeclass::TypeClass;
use std::collections::HashMap;

pub struct DragonType;

impl TypeClass for DragonType {
    fn key(&self) -> &'static str { "dragon" }
    fn description(&self) -> &'static str { "A fire-breathing monstrosity." }
    fn default_attributes(&self) -> HashMap<String, serde_json::Value> {
        HashMap::from([
            ("hp".into(), serde_json::json!(500)),
            ("breath".into(), serde_json::json!("fire")),
        ])
    }
    fn default_tags(&self) -> Vec<(String, String)> {
        vec![("aggressive".into(), "behavior".into())]
    }
}

registry.register_type(Arc::new(DragonType));
```

`WorldApi::create_character` reads the `character` type's defaults and
applies them on creation. For other types, apply defaults yourself if you
want them. A small helper does the trick:

```rust
async fn spawn_typed(
    world: &WorldApi,
    registry: &Registry,
    type_key: &str,
    name: &str,
    location: Option<UID>,
) -> Result<Object, &'static str> {
    let obj = world.create_object(type_key, name, location)
        .await.map_err(|_| "create failed")?;
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

## Commands

Source:
[`prelude.rs`](../muoxi/src/server/prelude.rs) (trait),
[`cmds.rs`](../muoxi/src/server/cmds.rs) (dispatcher).

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

The dispatcher resolves the first whitespace-delimited token against the
Registry (case-insensitive, alias-aware), evaluates `lock()` against the
actor's tags, then runs `execute_cmd`. The rest of the input arrives as
`ctx.args`.

The framework ships four commands: `look`, `say`, `quit`, and `who`. A
custom one looks like this:

```rust
use muoxi::prelude::{Command, CommandContext, CommandResult};
use muoxi::send;

#[derive(Debug)]
pub struct CmdShout;

#[async_trait]
impl Command for CmdShout {
    fn name(&self) -> &'static str { "shout" }
    fn aliases(&self) -> Vec<&'static str> { vec!["yell"] }

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

registry.register_command(Arc::new(CmdShout));
```

### Locks

A command's `lock()` returns an expression evaluated against the actor.
Source: [`locks.rs`](../muoxi/src/server/locks.rs).

Three forms today:

| Expression | Meaning |
| --- | --- |
| `"all()"` | Always allow (default) |
| `"false"` | Never allow |
| `"perm(NAME)"` | Actor must carry the tag `(NAME, "permission")` |

To grant a permission:

```rust
world.add_tag(character_uid, "admin", "permission").await?;
```

For richer logic, do the coarse gate in `lock()` and the rest inside
`execute_cmd`.

## Hooks

Lifecycle event listeners. Source:
[`hooks.rs`](../muoxi/src/server/hooks.rs).

```rust
pub struct HookContext<'a> {
    pub world: &'a WorldApi,
    pub session_uid: Option<UID>,
}

#[async_trait]
pub trait Hook: Send + Sync {
    fn name(&self) -> &'static str { "anonymous-hook" }

    async fn at_login(&self, ctx: &mut HookContext<'_>, account_uid: UID)
        -> HookResult { Ok(()) }
    async fn at_disconnect(&self, ctx: &mut HookContext<'_>, account_uid: UID)
        -> HookResult { Ok(()) }
    async fn at_object_created(&self, ctx: &mut HookContext<'_>, obj: &Object)
        -> HookResult { Ok(()) }
    async fn at_pre_destroy(&self, ctx: &mut HookContext<'_>, obj: &Object)
        -> HookResult { Ok(()) }
    async fn at_pre_move(&self, ctx: &mut HookContext<'_>, obj: &Object,
                         source: Option<UID>, dest: Option<UID>)
        -> HookResult { Ok(()) }
    async fn at_post_move(&self, ctx: &mut HookContext<'_>, obj: &Object,
                          source: Option<UID>, dest: Option<UID>)
        -> HookResult { Ok(()) }
    async fn at_say(&self, ctx: &mut HookContext<'_>, speaker: &Object,
                    message: &str)
        -> HookResult { Ok(()) }
}
```

`Hooks::emit` fires every listener in registration order, logging errors
at debug. `Hooks::emit_cancelable` short-circuits on the first `Err` —
the firing mode used by the `_pre_*` events when they're emitted.

Today the engine fires `at_login` and `at_disconnect`. The other methods
are part of the trait so downstream code can implement them now;
emission is being wired in over time — see the
[roadmap](roadmap.md).

```rust
use muoxi::hooks::{Hook, HookContext, HookResult};
use db::utils::UID;

pub struct AuditLogger;

#[async_trait]
impl Hook for AuditLogger {
    fn name(&self) -> &'static str { "audit-logger" }

    async fn at_login(&self, _ctx: &mut HookContext<'_>, acc: UID) -> HookResult {
        log::info!("login: account_uid={acc}");
        Ok(())
    }
    async fn at_disconnect(&self, _ctx: &mut HookContext<'_>, acc: UID) -> HookResult {
        log::info!("disconnect: account_uid={acc}");
        Ok(())
    }
}

registry.register_hook(Arc::new(AuditLogger));
```

## WorldApi

The DB facade your handlers should reach for. Source:
[`world.rs`](../muoxi/src/server/world.rs).

| Group | Methods |
| --- | --- |
| Objects | `create_object`, `get_object`, `move_object`, `contents_of` |
| Attributes | `set_attribute`, `get_attribute` |
| Tags | `add_tag`, `has_tag`, `objects_with_tag` |
| Accounts | `find_account_by_name`, `account_password_hash`, `create_account` |
| Characters | `list_account_characters`, `create_character`, `starting_room` |
| Escape hatch | `with_db` |

Methods return `QueryResult<T>` (Diesel's result type) or, for the
ones that can fail with descriptive messages, `Result<T, &'static str>`.

`with_db` gives you raw `&mut DatabaseHandler` for queries that don't
have a typed method yet. Keep its uses narrow; if you reach for it
repeatedly, add a typed method instead.

## Lower-level: object repos

If you're working on the framework itself, the typed repos in
[`db/src/objects/`](../db/src/objects/) sit underneath `WorldApi`:
`ObjectRepo`, `AttributeRepo`, `TagRepo`, `CharacterAccountRepo`. They
all take `&mut Conn`.

Command handlers and hooks should go through `WorldApi`. The repos are
for `db`-crate code and framework internals.

## SessionConfig and DEV_AUTOLOGIN

Not an extension point — a developer convenience.

Source: [`muoxi/src/lib.rs`](../muoxi/src/lib.rs).

```rust
pub struct SessionConfig {
    pub dev_autologin_room: Option<UID>,
}
```

When `dev_autologin_room` is `Some(uid)`, `process()` skips the auth
state machine and drops the connection into `Playing` as a throwaway
`Dev` character placed in that room. The env var `DEV_AUTOLOGIN`
controls it at startup.

## seed_world

A function to replace, not extend. Source:
[`seed.rs`](../muoxi/src/server/seed.rs).

```rust
pub async fn seed_world(world: &WorldApi)
    -> Result<UID, Box<dyn Error + Send + Sync>>
```

Idempotent on subsequent boots by looking for an existing object tagged
`(starting-room, system)`. First boot creates a single room named
"Limbo" with a stone and a goblin. Returns the room UID.

To replace it with your own starter content, call `seed_world` and then
your own seeder — or skip it entirely and tag your own room with
`(starting-room, system)` so `WorldApi::starting_room()` finds it.

## Worked example

```bash
cargo run --bin muoxi-example-extension
```

[`examples/extension/src/main.rs`](../examples/extension/src/main.rs)
registers a custom command and a custom type, constructs a `Registry`,
and runs a few resolve queries against it. It doesn't bind a listener —
copy `muoxi/src/server/main.rs` when you're ready to build a real server.
