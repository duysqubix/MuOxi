//! `say` — speak to characters in the current room.

use crate::prelude::{Command, CommandContext, CommandResult};
use crate::send;
use crate::world::WorldApi;
use async_trait::async_trait;
use db::utils::UID;
use std::collections::HashSet;

/// The `say` built-in command.
///
/// Fires `Hook::at_say` (via `Hooks::emit_cancelable`) after the speaker's
/// own echo but before broadcasting to room-mates. A hook returning `Err`
/// suppresses delivery to listeners while the speaker still sees their
/// local echo — matches the trait docstring's "Err suppresses delivery"
/// contract.
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

        let world_arc = ctx.world.clone();
        let world_ref: &WorldApi = world_arc.as_ref();
        let session_uid = ctx.client.uid;
        let speaker_for_hook = speaker_char.clone();
        let msg_for_hook = ctx.args.to_string();
        let suppress = ctx
            .registry
            .hooks
            .emit_cancelable(|h| {
                let speaker = speaker_for_hook.clone();
                let msg = msg_for_hook.clone();
                async move {
                    let mut hctx = crate::hooks::HookContext {
                        world: world_ref,
                        session_uid: Some(session_uid),
                    };
                    h.at_say(&mut hctx, &speaker, &msg).await
                }
            })
            .await
            .is_err();

        if suppress {
            return Ok(());
        }

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
