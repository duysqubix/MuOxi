//! In-process game-logic entry point.
//!
//! For v0.1 this is a thin pass-through to `cmds::dispatch`. The role of this
//! module is to be the obvious extension point for downstream MUDs that want
//! to add pre-/post-input processing (e.g., cooldown checks, idle timers,
//! global hook firing) without touching `states::execute`.

use crate::comms::Client;
use crate::prelude::LinesCodecResult;
use crate::registry::Registry;
use crate::world::WorldApi;
use std::sync::Arc;

/// Handle a single line of input from a `Playing` client.
pub async fn handle_input(
    client: &mut Client,
    registry: Arc<Registry>,
    world: Arc<WorldApi>,
    input: &str,
) -> LinesCodecResult<()> {
    crate::cmds::dispatch(client, registry, world, input, "Huh?").await;
    Ok(())
}
