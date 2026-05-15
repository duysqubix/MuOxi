//! Demonstrates registering a custom command and a custom TypeClass against
//! MuOxi. This is a reference for downstream MUD developers — it shows the
//! API shape but isn't a complete game server on its own.
//!
//! To embed in your own game server:
//! 1. Copy / fork the `muoxi_server` binary (`muoxi/src/server/main.rs`).
//! 2. Before the `TcpListener::bind` call, add your registrations:
//!
//! ```ignore
//! registry.register_command(Arc::new(CmdShout));
//! registry.register_type(Arc::new(DragonType));
//! ```
//!
//! 3. Run your fork instead of `muoxi_server`.

use async_trait::async_trait;
use db::DatabaseHandler;
use muoxi::prelude::{Command, CommandContext, CommandResult};
use muoxi::registry::Registry;
use muoxi::send;
use muoxi::typeclass::TypeClass;
use muoxi::world::WorldApi;
use std::collections::HashMap;
use std::sync::Arc;

/// Custom command: shouts the argument back in uppercase.
#[derive(Debug)]
pub struct CmdShout;

#[async_trait]
impl Command for CmdShout {
    fn name(&self) -> &'static str {
        "shout"
    }
    fn aliases(&self) -> Vec<&'static str> {
        vec!["yell"]
    }
    async fn execute_cmd(&self, ctx: CommandContext<'_>) -> CommandResult<()> {
        if ctx.args.is_empty() {
            let _ = send(ctx.client, "Shout what?").await;
            return Ok(());
        }
        let _ = send(
            ctx.client,
            &format!("You shout, \"{}\"!", ctx.args.to_uppercase()),
        )
        .await;
        Ok(())
    }
}

/// Custom in-world type: a fire-breathing dragon.
pub struct DragonType;

impl TypeClass for DragonType {
    fn key(&self) -> &'static str {
        "dragon"
    }
    fn description(&self) -> &'static str {
        "A fire-breathing monstrosity (downstream extension demo)"
    }
    fn default_attributes(&self) -> HashMap<String, serde_json::Value> {
        let mut m = HashMap::new();
        m.insert("hp".into(), serde_json::json!(500));
        m.insert("breath_attack".into(), serde_json::json!("fire"));
        m
    }
}

#[tokio::main]
async fn main() {
    println!(
        "muoxi-example-extension: registering a custom command + TypeClass against a Registry.\n\
         This example builds the registry but does not bind a TCP listener — that part is\n\
         delegated to a real `muoxi_server` fork that injects these registrations."
    );

    let world = Arc::new(WorldApi::new(DatabaseHandler::connect()));
    let registry = Arc::new(Registry::new(world));
    registry.register_builtin_types();
    registry.register_command(Arc::new(CmdShout));
    registry.register_type(Arc::new(DragonType));

    println!(
        "Registry now has built-in types + 'dragon' typeclass + 'shout'/'yell' command."
    );
    println!("Resolve test: 'shout HELLO' -> {:?}", registry.resolve_command("shout HELLO").map(|c| c.name()));
    println!("Resolve test: 'yell ow'    -> {:?}", registry.resolve_command("yell ow").map(|c| c.name()));
    println!("Resolve test: 'unknown'    -> {:?}", registry.resolve_command("unknown").map(|c| c.name()));
}
