#![allow(missing_docs)]

//! Command dispatcher used by the connection-state handler.

use crate::comms::{Client, Server};
use crate::prelude::CommandContext;
use crate::registry::Registry;
use crate::world::WorldApi;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Resolve and execute a single command line.
///
/// `input` is the raw line ("look at door"). The dispatcher looks up the
/// first whitespace-delimited token as a command name in the `Registry`.
/// If found, it runs the `lock` check and then `execute_cmd` with the rest
/// of the line as `ctx.args`. If not found, sends `unknown_msg` to the client.
pub async fn dispatch(
    client: &mut Client,
    registry: Arc<Registry>,
    world: Arc<WorldApi>,
    server: Arc<Mutex<Server>>,
    input: &str,
    unknown_msg: &str,
) {
    let Some(cmd) = registry.resolve_command(input) else {
        let _ = crate::send(client, unknown_msg).await;
        return;
    };

    if !crate::locks::check(&world, cmd.lock(), Some(client.uid)).await {
        let _ = crate::send(client, "You can't do that.").await;
        return;
    }

    let args = input
        .split_once(' ')
        .map(|(_, rest)| rest.trim())
        .unwrap_or("");

    let ctx = CommandContext {
        client,
        registry: registry.clone(),
        world: world.clone(),
        args,
        server,
    };
    if let Err(e) = cmd.execute_cmd(ctx).await {
        let _ = crate::send(client, &format!("Command error: {e}")).await;
    }
}
