# World building

How to populate your MUD with rooms, items, mobs, exits, and any custom
in-world type. This guide assumes you've read
[extension-guide.md](extension-guide.md) — in particular the TypeClass
section.

If you want to add a new *kind* of thing (a dragon, a vehicle), read the
extension guide first. This document is about the actual content of your
world: the rooms players walk through, the swords they pick up, the NPCs
they meet.

## The model in one paragraph

Every in-world thing is an `Object` row with a `type_key` discriminator.
Per-object freeform state lives in `object_attributes` (JSON values, keyed
by string). Searchable/groupable labels live in `object_tags` (a `(key,
category)` pair per tag). Containment is a self-referential FK:
`Object.location_uid` points at the room/container holding the object.

That's it. There's no separate `rooms` table, no separate `mobs` table.
Adding a custom type doesn't require a schema migration.

## Creating a room

```rust
use serde_json::json;

let town_square = world
    .create_object("room", "Town Square", None)
    .await?;
world.set_attribute(town_square.uid, "desc", json!(
    "Cobblestones worn smooth by generations. A wooden well stands at the \
     center. The smell of bread drifts from the bakery."
)).await?;
world.add_tag(town_square.uid, "safe-zone", "permission").await?;
```

The `location: None` argument means "this room isn't inside anything" — it
exists at the world level. Items and characters point at this room's UID
as their `location_uid`.

## Connecting rooms with exits

Exits are objects with `type_key = "exit"`. The convention is to store
the destination room's UID in a `destination` attribute on the exit, and
to place the exit object inside the source room.

```rust
let market = world.create_object("room", "Market District", None).await?;
world.set_attribute(market.uid, "desc", json!(
    "Stalls jostle for space. Hawkers shout prices in a dozen languages."
)).await?;

let exit_east = world
    .create_object("exit", "east", Some(town_square.uid))
    .await?;
world.set_attribute(exit_east.uid, "destination", json!(market.uid)).await?;

let exit_west = world
    .create_object("exit", "west", Some(market.uid))
    .await?;
world.set_attribute(exit_west.uid, "destination", json!(town_square.uid)).await?;
```

To make exits usable, you'll need to add a `go <direction>` (or
`<direction>` directly) command — see
[extension-guide.md § Commands](extension-guide.md#commands). The built-in
command set doesn't include movement; that's a decision for your MUD.

A reference implementation:

```rust
#[derive(Debug)]
pub struct CmdGo;

#[async_trait]
impl Command for CmdGo {
    fn name(&self) -> &'static str { "go" }
    fn aliases(&self) -> Vec<&'static str> { vec!["move"] }

    async fn execute_cmd(&self, ctx: CommandContext<'_>) -> CommandResult<()> {
        let direction = ctx.args.trim();
        if direction.is_empty() {
            let _ = send(ctx.client, "Go where?").await;
            return Ok(());
        }
        let Some(my_uid) = ctx.client.character_uid else {
            let _ = send(ctx.client, "You have no body to move.").await;
            return Ok(());
        };

        let me = ctx.world.get_object(my_uid).await
            .map_err(|_| "db error")?
            .ok_or("you don't seem to exist")?;
        let Some(here) = me.location_uid else {
            let _ = send(ctx.client, "You can't go anywhere from here.").await;
            return Ok(());
        };

        // find an exit named `direction` in this room
        let exits: Vec<_> = ctx.world.contents_of(here).await
            .map_err(|_| "db error")?
            .into_iter()
            .filter(|o| o.type_key == "exit" && o.name.eq_ignore_ascii_case(direction))
            .collect();
        let Some(exit) = exits.first() else {
            let _ = send(ctx.client, &format!("No exit '{direction}'.")).await;
            return Ok(());
        };

        let dest = ctx.world.get_attribute(exit.uid, "destination").await
            .map_err(|_| "db error")?
            .and_then(|v| v.as_i64());
        let Some(dest) = dest else {
            let _ = send(ctx.client, "That exit goes nowhere.").await;
            return Ok(());
        };

        ctx.world.move_object(my_uid, Some(dest)).await
            .map_err(|_| "db error")?;
        let _ = send(ctx.client, &format!("You go {direction}.")).await;
        Ok(())
    }
}
```

## Creating items

Items are objects with `type_key = "item"`. They live inside rooms, inside
containers, or inside characters (representing inventory).

```rust
let sword = world.create_object("item", "iron sword", Some(town_square.uid)).await?;
world.set_attribute(sword.uid, "desc", json!(
    "A plain iron blade. Functional, unadorned."
)).await?;
world.set_attribute(sword.uid, "weight", json!(3)).await?;
world.set_attribute(sword.uid, "damage", json!(8)).await?;
world.add_tag(sword.uid, "weapon", "kind").await?;
```

To make items pickable, you need `get` / `drop` commands. Same shape as
the movement example — read the actor's location, find the named object,
call `move_object` to set its `location_uid` to the character's UID
(inventory) or back to the room.

## Creating mobs

Mobs are objects with `type_key = "mob"`. They're indistinguishable from
characters at the persistence level — both go in `objects`. The difference
is that:

- Characters have a row in `character_accounts` linking them to a player account
- Mobs don't — they're owned by the world, not a player

```rust
let goblin = world.create_object("mob", "a tired-looking goblin", Some(town_square.uid)).await?;
world.set_attribute(goblin.uid, "hp", json!(15)).await?;
world.set_attribute(goblin.uid, "aggressive", json!(false)).await?;
world.set_attribute(goblin.uid, "loot_table", json!(["coin", "rusty_dagger"])).await?;
world.add_tag(goblin.uid, "creature", "kind").await?;
```

Making mobs *act* (wander, attack, decay over time) needs a tick loop — a
scheduler. That's on the roadmap; until then, run your own Tokio task at
startup that periodically calls `world.objects_with_tag(...)` to find
mobs and applies behavior.

## Tags vs attributes — when to use which

The rule of thumb:

- **Attributes**: per-object state that varies (HP, weight, gold count,
  current target). You read and write these constantly.
- **Tags**: categorical labels you look up *across* objects ("all rooms
  tagged `safe-zone`", "all objects with `weapon` kind", "everything in
  category `quest_item`"). The `object_tags` table is indexed on
  `(category, key)` so cross-object lookups are fast.

If you find yourself doing `objects.iter().filter(|o| attr(o, "X") == "Y")`
in Rust, that's a tag. Move it to `object_tags` and use
`world.objects_with_tag("Y", "X")` instead.

## Locations: a brief discussion

`Object.location_uid` is a self-referential FK. This means:

- A character has location `Some(town_square.uid)` when standing in the
  square.
- An item has location `Some(character.uid)` when in inventory, or
  `Some(town_square.uid)` when on the ground.
- A room has location `None` (rooms aren't usually contained by other
  objects, though you can have nested zones if you want — set
  `location_uid` to a region/zone object).
- An exit lives in the room it leads out of (its `location_uid` is the
  source room) and carries a `destination` attribute pointing at the
  target room.

Loops in the location graph are technically possible at the schema level
but logically broken — don't put room A inside item B inside room A.
The FK is `ON DELETE SET NULL`, so deleting a container reparents its
contents to `None` rather than cascading.

## Permissions: the `perm()` lock pattern

To grant a permission to a character, tag them with the permission name
in the `permission` category:

```rust
world.add_tag(admin_character.uid, "admin", "permission").await?;
world.add_tag(admin_character.uid, "builder", "permission").await?;
```

Then a command with `fn lock(&self) -> &'static str { "perm(builder)" }`
will allow only characters carrying the `builder` permission tag.

This is the minimum-viable role/perm system. For richer logic (multiple
required perms, role hierarchies, time-bound grants), put the gating
inside `execute_cmd` after a coarse `perm(...)` check.

## Replacing the default seed

The framework calls `seed_world()` unconditionally at server startup to
guarantee a starting room exists. If you want a richer initial world,
chain your own seeder after it in your `main()`:

```rust
let starting_room = seed_world(&world).await?;
build_my_starter_zone(&world, starting_room).await?;
```

Or replace it entirely. The contract is: when a new character is created
via `MainMenu` → `new <name>`, the framework calls
`world.starting_room()` to find where to place them. That's just a lookup
for the object tagged `(starting-room, system)`. So if your custom seeder
tags some room of yours with that pair, character creation will pick it up.

```rust
let throne_room = world.create_object("room", "The Throne Room", None).await?;
world.set_attribute(throne_room.uid, "desc", json!(
    "Gilt and marble. The high seat sits empty."
)).await?;
world.add_tag(throne_room.uid, "starting-room", "system").await?;
```

If both `seed_world`'s "Limbo" AND your `throne_room` carry that tag,
`world.starting_room()` returns whichever comes first by UID. Pick one
strategy — either delete Limbo after seeding your zone, or never call
`seed_world()`.

## Where to next

- [extension-guide.md](extension-guide.md) — register custom `TypeClass`es,
  `Command`s, `Hook`s
- [architecture.md](architecture.md) — the object/attribute/tag model
  rationale
- [roadmap.md](roadmap.md) — what's missing (scheduler, broadcast,
  generic TypeClass auto-apply)
