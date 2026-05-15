//! `who` — list connected sessions.

use crate::prelude::{Command, CommandContext, CommandResult};
use crate::send;
use async_trait::async_trait;

/// The `who` built-in command.
#[derive(Debug)]
pub struct CmdWho;

#[async_trait]
impl Command for CmdWho {
    fn name(&self) -> &'static str {
        "who"
    }

    async fn execute_cmd(&self, ctx: CommandContext<'_>) -> CommandResult<()> {
        let _ = send(ctx.client, "Players online: (server-aware listing lands with the auth state machine)").await;
        Ok(())
    }
}
