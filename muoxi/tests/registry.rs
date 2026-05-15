//! Smoke tests for `Registry` behaviour. The DB connection is in-memory; the
//! tests don't actually touch the DB but `Registry::new` requires an
//! `Arc<WorldApi>` which wraps a real `DatabaseHandler`.

use async_trait::async_trait;
use db::DatabaseHandler;
use muoxi::prelude::{Command, CommandContext, CommandResult};
use muoxi::registry::Registry;
use muoxi::typeclass::TypeClass;
use muoxi::typeclass::builtins::{CharacterType, RoomType};
use muoxi::world::WorldApi;
use std::sync::{Arc, Once};

static SETUP: Once = Once::new();

fn make_registry() -> Registry {
    SETUP.call_once(|| {
        unsafe {
            std::env::set_var("DATABASE_URL", ":memory:");
        }
    });
    let world = Arc::new(WorldApi::new(DatabaseHandler::connect()));
    Registry::new(world)
}

#[derive(Debug)]
struct CmdEcho;

#[async_trait]
impl Command for CmdEcho {
    fn name(&self) -> &'static str {
        "echo"
    }
    fn aliases(&self) -> Vec<&'static str> {
        vec!["e"]
    }
    async fn execute_cmd(&self, _ctx: CommandContext<'_>) -> CommandResult<()> {
        Ok(())
    }
}

#[tokio::test]
async fn registers_and_resolves_command_by_name_and_alias() {
    let r = make_registry();
    r.register_command(Arc::new(CmdEcho));

    assert!(r.resolve_command("echo").is_some());
    assert!(r.resolve_command("ECHO").is_some());
    assert!(r.resolve_command("e").is_some());
    assert!(r.resolve_command("echo something").is_some());
    assert!(r.resolve_command("nothing").is_none());
}

#[tokio::test]
async fn registers_typeclasses_and_looks_up_by_key() {
    let r = make_registry();
    r.register_type(Arc::new(CharacterType));
    r.register_type(Arc::new(RoomType));

    assert!(r.get_type("character").is_some());
    assert!(r.get_type("room").is_some());
    assert!(r.get_type("nope").is_none());
    assert_eq!(r.get_type("character").unwrap().key(), "character");
}

#[tokio::test]
async fn builtin_typeclass_default_attributes_present() {
    let t = CharacterType;
    let attrs = t.default_attributes();
    assert!(attrs.contains_key("hp"));
    assert!(attrs.contains_key("desc"));
}
