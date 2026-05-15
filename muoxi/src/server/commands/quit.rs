//! `quit` — end the session. Actual disconnect is handled by `states::execute`
//! reading the trimmed input directly; this command just emits a farewell line.

use crate::prelude::{Command, CommandContext, CommandResult};
use crate::send;
use async_trait::async_trait;

/// The `quit` built-in command.
#[derive(Debug)]
pub struct CmdQuit;

#[async_trait]
impl Command for CmdQuit {
    fn name(&self) -> &'static str {
        "quit"
    }
    fn aliases(&self) -> Vec<&'static str> {
        vec!["q", "exit"]
    }

    async fn execute_cmd(&self, ctx: CommandContext<'_>) -> CommandResult<()> {
        let _ = send(ctx.client, "Goodbye.").await;
        Ok(())
    }
}
