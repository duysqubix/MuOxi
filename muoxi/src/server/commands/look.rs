//! `look` — describe the current location.

use crate::prelude::{Command, CommandContext, CommandResult};
use crate::send;
use async_trait::async_trait;

/// The `look` built-in command.
#[derive(Debug)]
pub struct CmdLook;

#[async_trait]
impl Command for CmdLook {
    fn name(&self) -> &'static str {
        "look"
    }
    fn aliases(&self) -> Vec<&'static str> {
        vec!["l"]
    }

    async fn execute_cmd(&self, ctx: CommandContext<'_>) -> CommandResult<()> {
        let Some(my_uid) = ctx.client.character_uid else {
            let _ = send(ctx.client, "You don't have a character to look around with.").await;
            return Ok(());
        };
        let me = ctx
            .world
            .get_object(my_uid)
            .await
            .map_err(|_| "db error")?;
        let me = me.ok_or("you don't seem to exist")?;

        let location_uid = match me.location_uid {
            Some(uid) => uid,
            None => {
                let _ = send(ctx.client, "You are floating in the void.").await;
                return Ok(());
            }
        };

        let room = ctx
            .world
            .get_object(location_uid)
            .await
            .map_err(|_| "db error")?;
        let room = room.ok_or("location missing")?;
        let desc = ctx
            .world
            .get_attribute(room.uid, "desc")
            .await
            .map_err(|_| "db error")?
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "You see nothing special.".to_string());

        let contents = ctx
            .world
            .contents_of(room.uid)
            .await
            .map_err(|_| "db error")?;
        let visible: Vec<String> = contents
            .into_iter()
            .filter(|o| o.uid != my_uid)
            .map(|o| format!("  {}", o.name))
            .collect();
        let here_block = if visible.is_empty() {
            String::new()
        } else {
            format!("\nHere you see:\n{}", visible.join("\n"))
        };

        let _ = send(
            ctx.client,
            &format!("[{}]\n{}{}", room.name, desc, here_block),
        )
        .await;
        Ok(())
    }
}
