//! `say` — speak in the current room.

use crate::prelude::{Command, CommandContext, CommandResult};
use crate::send;
use async_trait::async_trait;

/// The `say` built-in command.
#[derive(Debug)]
pub struct CmdSay;

#[async_trait]
impl Command for CmdSay {
    fn name(&self) -> &'static str {
        "say"
    }
    fn aliases(&self) -> Vec<&'static str> {
        vec!["'", "\""]
    }

    async fn execute_cmd(&self, ctx: CommandContext<'_>) -> CommandResult<()> {
        if ctx.args.is_empty() {
            let _ = send(ctx.client, "Say what?").await;
            return Ok(());
        }
        let _ = send(ctx.client, &format!("You say, \"{}\"", ctx.args)).await;
        Ok(())
    }
}
