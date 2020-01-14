//!
//! Definitions for CommandSets. Depending on a variety of factors, you have access
//! to different sets of commands. Some of the basic conditions ruling this would be:
//!
//! * Connection State
//! * Roles
//!
use crate::comms::Client;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use tokio::sync::mpsc;
use tokio_util::codec::LinesCodecError;

/// alias for sending channel
pub type Tx = mpsc::UnboundedSender<String>;

/// alias for recieving channel
pub type Rx = mpsc::UnboundedReceiver<String>;

/// Result generic resulting with decoding/encoding errors
pub type LinesCodecResult<T> = Result<T, LinesCodecError>;

/// Custom error type revolving around the success of executing commands
pub type CommandResult<T> = Result<T, &'static str>;

/// Current listening port of the MuOxi game engine
pub static GAME_ADDR: &'static str = "127.0.0.1:4567";

/// Current listening port of the staging proxy server
pub static PROXY_ADDR: &'static str = "127.0.0.1:8000";

/// defines a command trait
pub trait Command {
    /// name of command
    fn name(&self) -> String;

    /// a list of aliases that will invoke command
    fn aliases(&self) -> Vec<&str>;

    /// execute the actual command but only directs commands to game_server,
    /// will err if client state is not in playing
    fn execute_cmd(&self, game_server: &mut Client) -> CommandResult<()>;
}

/// defines a common collection of commands
#[derive(Debug, Clone)]
pub struct CmdSet<T: Command + Debug + Hash + Eq>(pub HashSet<T>);
