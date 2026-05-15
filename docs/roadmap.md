# Roadmap

This doc describes where MuOxi is headed, in broad strokes. It's not a
release plan — there are no version dates, no feature commitments, no
guarantees about ordering. It's a map of the terrain.

## Where we are

MuOxi runs end-to-end. A player can connect, register, create a
character, and walk into the world. State persists across restarts. The
extension surface (Registry, TypeClass, Command, Hook, WorldApi) is in
place and exercised by the built-in command set and the example crate.

It is, deliberately, a starting point — a small, opinionated kernel
rather than a full game.

## Direction

A few themes shape the work ahead.

**Completing the lifecycle.** Several hook events are exposed by the
trait but not yet emitted by the engine. Closing those gaps and making
the extension surface uniformly reactive is a near-term focus.

**Sharper extension ergonomics.** The current Registry + Command + Hook
shape works, but a few rough edges show up when you build on it: a more
expressive lock language, automatic application of type-class defaults,
clearer paths to broadcast or co-locate messages between players.

**Authentication beyond the basics.** The login flow is functional; what
it doesn't yet support is pluggable backends, richer password policies,
or admin tooling. Those land when the demand is real.

**A second process.** The current single-binary topology is fine for
small worlds. A separate proxy and engine, talking over a framed
protocol, would unlock hot-reloading of game logic without dropping
players. This is on the horizon, not the calendar.

**Ambient world.** Timed behaviour — mob AI ticks, decay timers,
weather, scheduled events — is missing today and needed for any
non-trivial MUD. A scheduler that survives restarts is the natural
shape.

These themes overlap. Work happens where the friction is highest at the
time.

## What this framework won't be

MuOxi is infrastructure, not a game.

We don't ship a combat system, a magic system, an economy, a quest
engine, a default world, or a permissions hierarchy beyond the minimum
needed to gate commands. Every MUD has strong opinions about these
things, and every opinion is incompatible with somebody else's. The
framework's job is to stay quiet on them.

It also won't grow into an admin web app, a content-management UI, or
a level editor. Those are reasonable things to build, and they're
reasonable things to build *on top of* MuOxi. They aren't its concern.

If something feels missing and you don't see it on the horizon, open an
issue. The shape of this project bends to real use cases; it doesn't
bend to hypothetical ones.
