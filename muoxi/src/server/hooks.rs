//! Lifecycle hooks. Implementors register handlers; the engine fires events.

use crate::world::WorldApi;
use async_trait::async_trait;
use db::objects::Object;
use db::utils::UID;
use std::sync::Arc;

/// Result of a hook invocation. `Err(reason)` cancels the in-progress action
/// (where applicable — see each method's docs).
pub type HookResult = Result<(), &'static str>;

/// Read-mostly context handed to hook implementations.
pub struct HookContext<'a> {
    /// shared world facade
    pub world: &'a WorldApi,
    /// session UID if a session is associated; None for system events
    pub session_uid: Option<UID>,
}

/// Hook trait. All methods have default no-op impls — implementers override
/// only what they need.
#[async_trait]
pub trait Hook: Send + Sync {
    /// Friendly identifier (used for diagnostics / logs).
    fn name(&self) -> &'static str {
        "anonymous-hook"
    }

    /// After a successful login. `account_uid` is the login account.
    async fn at_login(&self, _ctx: &mut HookContext<'_>, _account_uid: UID) -> HookResult {
        Ok(())
    }

    /// Before a clean disconnect or detected drop.
    async fn at_disconnect(&self, _ctx: &mut HookContext<'_>, _account_uid: UID) -> HookResult {
        Ok(())
    }

    /// After an `Object` is created (called by `WorldApi::create_object`'s caller).
    async fn at_object_created(&self, _ctx: &mut HookContext<'_>, _obj: &Object) -> HookResult {
        Ok(())
    }

    /// Before an object is destroyed. Returning `Err` cancels the deletion.
    async fn at_pre_destroy(&self, _ctx: &mut HookContext<'_>, _obj: &Object) -> HookResult {
        Ok(())
    }

    /// Before a move. Returning `Err` cancels the move.
    async fn at_pre_move(
        &self,
        _ctx: &mut HookContext<'_>,
        _obj: &Object,
        _source: Option<UID>,
        _destination: Option<UID>,
    ) -> HookResult {
        Ok(())
    }

    /// After a successful move.
    async fn at_post_move(
        &self,
        _ctx: &mut HookContext<'_>,
        _obj: &Object,
        _source: Option<UID>,
        _destination: Option<UID>,
    ) -> HookResult {
        Ok(())
    }

    /// After an object says something. Returning `Err` suppresses delivery.
    async fn at_say(
        &self,
        _ctx: &mut HookContext<'_>,
        _speaker: &Object,
        _message: &str,
    ) -> HookResult {
        Ok(())
    }
}

/// Collection of registered hooks. Fires events to all registered listeners
/// in registration order; the first `Err` short-circuits cancelable events.
#[derive(Default, Clone)]
pub struct Hooks {
    inner: Arc<parking_lot::RwLock<Vec<Arc<dyn Hook>>>>,
}

impl Hooks {
    /// Construct an empty hook collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a hook listener. Listeners are invoked in registration order.
    pub fn register(&self, hook: Arc<dyn Hook>) {
        self.inner.write().push(hook);
    }

    /// Fire a non-cancelable event by calling `f` on each hook in turn.
    pub async fn emit<F, Fut>(&self, mut f: F)
    where
        F: FnMut(Arc<dyn Hook>) -> Fut,
        Fut: std::future::Future<Output = HookResult>,
    {
        let snapshot: Vec<_> = self.inner.read().clone();
        for h in snapshot {
            if let Err(reason) = f(h.clone()).await {
                log::debug!("hook {:?} returned err: {reason}", h.name());
            }
        }
    }

    /// Fire a cancelable event. Returns first `Err` encountered, or `Ok` if all pass.
    pub async fn emit_cancelable<F, Fut>(&self, mut f: F) -> HookResult
    where
        F: FnMut(Arc<dyn Hook>) -> Fut,
        Fut: std::future::Future<Output = HookResult>,
    {
        let snapshot: Vec<_> = self.inner.read().clone();
        for h in snapshot {
            f(h.clone()).await?;
        }
        Ok(())
    }
}
