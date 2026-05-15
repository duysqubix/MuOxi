//! In-process game-logic entry point.
//!
//! When a session reaches [`ConnStates::Playing`](crate::states::ConnStates::Playing),
//! per-line input is dispatched here. For v0.1 this is a placeholder that
//! echoes the input prefixed with `"Game > "`; downstream framework users
//! replace or extend this module.
//!
//! Future direction (Plan 4): this module will host the world API surface
//! that scripts and command handlers call into.

use crate::comms::Client;
use crate::prelude::LinesCodecResult;
use crate::send;

/// Handle a single line of input from a `Playing` client.
pub async fn handle_input(client: &mut Client, input: &str) -> LinesCodecResult<()> {
    let response = format!("Game > {}", input);
    send(client, &response).await
}
