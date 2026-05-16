//! `heartbeat` — emits a log line every tick. Demonstrates the handler shape.
//!
//! Persistent state field `count`: total ticks since creation. Useful for
//! verifying scheduler liveness in CI.

use crate::scheduler::{ScriptContext, ScriptHandler};
use async_trait::async_trait;
use db::utils::UID;

/// Built-in handler that increments a tick counter in its persisted state.
#[derive(Debug)]
pub struct HeartbeatHandler;

#[async_trait]
impl ScriptHandler for HeartbeatHandler {
    fn key(&self) -> &'static str {
        "heartbeat"
    }

    async fn run(
        &self,
        _ctx: &mut ScriptContext<'_>,
        _object_uid: Option<UID>,
        state: serde_json::Value,
    ) -> Result<serde_json::Value, &'static str> {
        let count = state
            .get("count")
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
            + 1;
        log::info!(target: "muoxi::heartbeat", "tick {}", count);
        Ok(serde_json::json!({ "count": count }))
    }
}
