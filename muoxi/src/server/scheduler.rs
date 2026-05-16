//! Persistent scheduler for `Script` rows.
//!
//! `Scheduler::run` is the long-lived background task. It polls the DB every
//! `POLL_INTERVAL` ms for due scripts, resolves their `handler_key` against
//! the `Registry`, and calls `ScriptHandler::run`. Successful runs advance
//! `next_run_at`; errors disable the script.

use crate::registry::Registry;
use crate::world::WorldApi;
use async_trait::async_trait;
use db::utils::UID;
use std::sync::Arc;
use std::time::Duration;

/// Per-run context passed to `ScriptHandler::run`.
pub struct ScriptContext<'a> {
    /// the world facade (DB access)
    pub world: &'a WorldApi,
    /// the registry (so handlers can look up other handlers, types, etc.)
    pub registry: Arc<Registry>,
}

/// Implement this for any periodic / scheduled behavior.
///
/// The framework provides one built-in handler (`HeartbeatHandler`) as a
/// worked example; downstream MUDs register their own.
#[async_trait]
pub trait ScriptHandler: Send + Sync {
    /// Stable identifier — also stored in `scripts.handler_key`.
    fn key(&self) -> &'static str;

    /// Execute one tick. Receive the script's persistent JSON state, return
    /// the new state to persist. Return `Err` to disable the script.
    async fn run(
        &self,
        ctx: &mut ScriptContext<'_>,
        object_uid: Option<UID>,
        state: serde_json::Value,
    ) -> Result<serde_json::Value, &'static str>;
}

/// How often the scheduler scans the DB for due jobs.
const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// The scheduler task. Spawn one of these per server instance.
pub struct Scheduler {
    registry: Arc<Registry>,
}

impl Scheduler {
    /// Construct from a shared `Registry`. The scheduler dispatches through
    /// the registry's `world` facade and its `script_handler` lookup.
    pub fn new(registry: Arc<Registry>) -> Self {
        Self { registry }
    }

    /// Run forever. Cancel by dropping the spawned `JoinHandle`.
    pub async fn run(self) {
        loop {
            if let Err(e) = self.tick().await {
                log::warn!("scheduler tick error: {e}");
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    }

    async fn tick(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let due = self
            .registry
            .world
            .with_db(|db| db.scripts.list_due(&mut db.handle))
            .await?;

        for script in due {
            let handler = match self.registry.script_handler(&script.handler_key) {
                Some(h) => h,
                None => {
                    log::warn!(
                        "script {} references unregistered handler {:?}; disabling",
                        script.id,
                        script.handler_key
                    );
                    self.registry
                        .world
                        .with_db(|db| db.scripts.disable(&mut db.handle, script.id))
                        .await
                        .ok();
                    continue;
                }
            };

            let parsed_state: serde_json::Value =
                serde_json::from_str(&script.state).unwrap_or(serde_json::json!({}));

            let mut ctx = ScriptContext {
                world: &self.registry.world,
                registry: self.registry.clone(),
            };

            match handler
                .run(&mut ctx, script.object_uid, parsed_state)
                .await
            {
                Ok(new_state) => {
                    self.registry
                        .world
                        .with_db(|db| db.scripts.record_run(&mut db.handle, script.id, &new_state))
                        .await
                        .ok();
                }
                Err(reason) => {
                    log::warn!(
                        "script {} (handler {:?}) returned err: {reason}; disabling",
                        script.id,
                        script.handler_key
                    );
                    self.registry
                        .world
                        .with_db(|db| db.scripts.disable(&mut db.handle, script.id))
                        .await
                        .ok();
                }
            }
        }
        Ok(())
    }
}
