//! Built-in script handlers. Downstream MUDs register their own via
//! `Registry::register_script_handler`.

pub mod heartbeat;

use crate::registry::Registry;
use std::sync::Arc;

/// Register every built-in script handler with `registry`.
pub fn register_all(registry: &Registry) {
    registry.register_script_handler(Arc::new(heartbeat::HeartbeatHandler));
}
