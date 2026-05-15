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
        let mut listed: Vec<String> = ctx
            .world
            .with_db(|db| {
                use db::diesel::prelude::*;
                use db::schema::objects::dsl;
                dsl::objects
                    .filter(dsl::type_key.eq("character"))
                    .select(dsl::name)
                    .load::<String>(&mut db.handle)
                    .unwrap_or_default()
            })
            .await;
        listed.sort();
        listed.dedup();
        let body = if listed.is_empty() {
            "No characters in the world yet.".to_string()
        } else {
            format!("Characters in the world ({}):\n  {}", listed.len(), listed.join("\n  "))
        };
        let _ = send(ctx.client, &body).await;
        Ok(())
    }
}
