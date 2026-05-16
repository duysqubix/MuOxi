//! `say` — speak to characters in the current room.

use crate::prelude::{Command, CommandContext, CommandResult};
use crate::send;
use async_trait::async_trait;
use db::utils::UID;
use std::collections::HashSet;

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

        let Some(speaker_char_uid) = ctx.client.character_uid else {
            let _ = send(ctx.client, "You have no body to speak with.").await;
            return Ok(());
        };

        let speaker_char = match ctx.world.get_object(speaker_char_uid).await {
            Ok(Some(obj)) => obj,
            _ => {
                let _ = send(ctx.client, "Your character seems to have vanished.").await;
                return Ok(());
            }
        };

        let Some(room) = speaker_char.location_uid else {
            let _ = send(ctx.client, "There is nothing here to speak into.").await;
            return Ok(());
        };

        let _ = send(ctx.client, &format!("You say, \"{}\"", ctx.args)).await;

        let in_room = match ctx.world.contents_of(room).await {
            Ok(v) => v,
            Err(_) => return Ok(()),
        };
        let listeners: HashSet<UID> = in_room
            .into_iter()
            .filter(|o| o.type_key == "character" && o.uid != speaker_char_uid)
            .map(|o| o.uid)
            .collect();

        if listeners.is_empty() {
            return Ok(());
        }

        let message = format!("{} says, \"{}\"", speaker_char.name, ctx.args);
        let mut server = ctx.server.lock().await;
        for comms in server.clients.values_mut() {
            if let Some(char_uid) = comms.character_uid {
                if listeners.contains(&char_uid) {
                    let _ = comms.tx.send(message.clone());
                }
            }
        }

        Ok(())
    }
}
