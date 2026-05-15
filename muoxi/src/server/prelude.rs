//! Command trait + per-command context handed to handler invocations.
//!
//! Commands are registered against the central `Registry` and resolved by
//! name/alias at runtime. Each invocation receives a `CommandContext` carrying
//! the session, the registry, the world facade, and the raw argument string.

use crate::comms::Client;
use crate::registry::Registry;
use crate::world::WorldApi;
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::codec::LinesCodecError;

/// outbound channel handle stored in `Comms`
pub type Tx = mpsc::UnboundedSender<String>;

/// inbound channel handle held by `Client`
pub type Rx = mpsc::UnboundedReceiver<String>;

/// result type for codec helpers
pub type LinesCodecResult<T> = Result<T, LinesCodecError>;

/// result type for command handlers
pub type CommandResult<T> = Result<T, &'static str>;

/// Per-invocation context handed to each command's `execute_cmd`.
pub struct CommandContext<'a> {
    /// the client session this command runs against
    pub client: &'a mut Client,
    /// the framework's registry (lookup other commands, types, fire hooks)
    pub registry: Arc<Registry>,
    /// world facade (DB access)
    pub world: Arc<WorldApi>,
    /// raw arguments after the command name (may be empty)
    pub args: &'a str,
}

/// A registered command. Implementations are unit structs (or carry only
/// configuration) — runtime state lives in `Client` or world.
#[async_trait]
pub trait Command: Debug + Send + Sync {
    /// Primary command name (lower-case).
    fn name(&self) -> &'static str;

    /// Aliases that also invoke this command. Default empty.
    fn aliases(&self) -> Vec<&'static str> {
        Vec::new()
    }

    /// Lock expression (see `crate::locks::check`). Default: `"all()"`.
    fn lock(&self) -> &'static str {
        "all()"
    }

    /// Execute against `ctx`.
    async fn execute_cmd(&self, ctx: CommandContext<'_>) -> CommandResult<()>;
}
