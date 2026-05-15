//! Definitions for `Command` / `CmdSet`. Depending on a variety of factors
//! (connection state, roles, etc.) a client gets a different set of commands.

use crate::comms::Client;
use async_trait::async_trait;
use std::fmt::Debug;
use std::marker::{Send, Sync};
use tokio::sync::mpsc;
use tokio_util::codec::LinesCodecError;

#[macro_export]
/// Build a `CmdSet` from `impl Command` values.
macro_rules! cmdset {
    ($($cmd: expr),+) => {
        {
            let mut cmds: Vec<Box<dyn Command + Send>> = Vec::new();
            $(
                cmds.push(Box::new($cmd));
            )*
            CmdSet::new(cmds)
        }
    };
}

/// alias for sending channel
pub type Tx = mpsc::UnboundedSender<String>;

/// alias for recieving channel
pub type Rx = mpsc::UnboundedReceiver<String>;

/// Result generic resulting with decoding/encoding errors
pub type LinesCodecResult<T> = Result<T, LinesCodecError>;

/// Custom error type revolving around the success of executing commands
pub type CommandResult<T> = Result<T, &'static str>;

/// Defines a command. All command logic must live inside the trait impl,
/// because dispatch goes through `Box<dyn Command + Send>` trait objects.
#[async_trait]
pub trait Command: Debug + Sync {
    /// Primary command name (lower-case).
    fn name(&self) -> &str;

    /// Aliases that also invoke this command.
    fn aliases(&self) -> Vec<&str>;

    /// Execute the command on the supplied client.
    async fn execute_cmd(&self, game_server: &mut Client) -> CommandResult<()>;

    /// Returns true when `cmd` matches the command's name or any alias.
    fn is_match(&self, cmd: &str) -> bool {
        let cmd = cmd.to_lowercase();
        if cmd == self.name() {
            return true;
        }
        self.aliases().iter().any(|c| *c == cmd)
    }
}

/// Collection of `Box<dyn Command + Send>`.
///
/// All command logic must live within the `Command` trait impl, because dispatch
/// goes through trait objects. Storing object-specific fields on a unit-struct
/// command is meaningless - the compiler can't see them through the trait object.
#[derive(Debug)]
pub struct CmdSet {
    /// holds a list of valid commands in set
    pub cmds: Vec<Box<dyn Command + Send>>,
}

impl CmdSet {
    /// Create a new command set.
    pub fn new(cmds: Vec<Box<dyn Command + Send>>) -> Self {
        Self { cmds }
    }

    /// Find the command matching `cmd_string`. Returns `None` if no match.
    pub fn get(&mut self, cmd_string: String) -> Option<&mut (dyn Command + Send)> {
        for cmd in self.cmds.iter_mut() {
            if cmd.is_match(&cmd_string) {
                return Some(cmd.as_mut());
            }
        }
        None
    }
}
