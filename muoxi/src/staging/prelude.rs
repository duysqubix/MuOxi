//!
//! Definitions for CommandSets. Depending on a variety of factors, you have access
//! to different sets of commands. Some of the basic conditions ruling this would be:
//!
//! * Connection State
//! * Roles
//!
use crate::comms::Client;
use async_trait::async_trait;
use std::fmt::Debug;
use std::marker::{Send, Sync};
use tokio::sync::mpsc;
use tokio_util::codec::LinesCodecError;

#[macro_export]
/// an easier way to create a command set from
/// valid `impl Command` objects
macro_rules! cmdset {
    ($($cmd: expr),+) => {
        {
            let mut cmds: Vec<Box<dyn Command+Send>> = Vec::new();
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

/// Current listening port of the MuOxi game engine
pub static GAME_ADDR: &'static str = "127.0.0.1:4567";

/// Current listening port of the staging proxy server
pub static PROXY_ADDR: &'static str = "127.0.0.1:8000";

/// defines a command trait
#[async_trait]
pub trait Command: Debug + Sync {
    /// name of command
    fn name(&self) -> &str;

    /// a list of aliases that will invoke command
    fn aliases(&self) -> Vec<&str>;

    /// execute the actual command but only directs commands to game_server,
    /// will err if client state is not in playing
    async fn execute_cmd(&self, game_server: &mut Client) -> CommandResult<()>;

    /// tests to see if supplied string is a valid command
    fn is_match<'a>(&self, cmd: &'a str) -> bool {
        let cmd = cmd.to_lowercase();
        // first check to see if there is a match with the name itself
        if cmd == self.name() {
            return true;
        }

        if self.aliases().len() > 0 {
            for c in self.aliases().iter() {
                if *c == cmd {
                    return true;
                }
            }
        }
        false
    }
}

/// defines a common collection of commands
/// I really don't like how this is made. It is super messy
/// and in a few months I'd probably look at this and be like
/// WTF... Essentially this struct holds a list of commands, but the
/// objects contained are dynamic, but they all inherit the Commands trait
/// Essentially I have heap allocated Vector that holds trait objects, and the
/// `CmdSet::get()` method uses a messy logic flow to extract the appropriate cmd
/// from the user input... probably need to work on this eventually and rework it.
#[derive(Debug)]
pub struct CmdSet {
    /// holds a list of valid commands in set
    pub cmds: Vec<Box<dyn Command + Send>>,
}

impl CmdSet {
    /// create a new command set based on appropriate structs that implement Command Trait
    pub fn new(cmds: Vec<Box<dyn Command + Send>>) -> Self {
        Self { cmds: cmds }
    }

    /// check to see if command exists within CmdSet
    /// and returns the dyn Command that it matches with
    /// this is really *fucking messy*
    pub fn get(&mut self, cmd_string: String) -> Option<&mut (dyn Command + Send)> {
        let mut cntr = 0;
        for cmd in self.cmds.iter_mut() {
            if cmd.is_match(&cmd_string) {
                break;
            }
            cntr += 1;
        }

        if let Some(cmd) = self.cmds.get_mut(cntr) {
            return Some(&mut **cmd);
        } else {
            return None;
        }
    }
}
