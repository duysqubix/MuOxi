//! Built-in commands. Downstream framework users register their own via
//! `Registry::register_command`.

pub mod look;
pub mod quit;
pub mod say;
pub mod who;

use crate::registry::Registry;
use std::sync::Arc;

/// Register every built-in command with `registry`.
pub fn register_all(registry: &Registry) {
    registry.register_command(Arc::new(look::CmdLook));
    registry.register_command(Arc::new(say::CmdSay));
    registry.register_command(Arc::new(quit::CmdQuit));
    registry.register_command(Arc::new(who::CmdWho));
}
