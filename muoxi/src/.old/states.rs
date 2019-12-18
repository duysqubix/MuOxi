//!
//! File: states.rs
//! Usage: defines all the states a player can be in, and ensures seemless transition between states
//!
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LinesCodec};

///
/// Universal initialization trait that each State will *inherit*
/// This will run once upon entering a new state
///
trait Process {
    fn process(&mut self) -> Result<(), String>;
}

pub enum PlayerType {
    NewPlayer(String),
    ExistingPlayer(String),
}
///
/// A structure to hold commands available to player depending on the ConnState
/// @TODO This needs to be replaced by a hanlder to a set of commands, not in this file
pub struct CmdSet;

impl CmdSet {
    fn new() -> Self {
        CmdSet
    }
}
///
/// The state machine that will handle a players progression between the different
/// connectivity states.
///
pub struct ConnState<T> {
    client: Framed<TcpStream, LinesCodec>,
    cmdset: CmdSet,
    state: T,
}

impl<T: Process> Process for ConnState<T> {
    fn process(&mut self) -> Result<(), String> {
        self.client
            .send("Enter player name, or create `new`".to_string())
            .await;
        self.state.process().unwrap();
        Ok(())
    }
}

impl ConnState<EnterGame> {
    pub fn new(stream: Framed<TcpStream, LinesCodec>) -> Self {
        println!("entered game");
        ConnState {
            client: stream,
            cmdset: CmdSet::new(),
            state: EnterGame {},
        }
    }
}

///
/// Upon successful connection to game server
/// connected client will enter `EnterGame` state
/// as initial ConnState
///
pub struct EnterGame {}

impl Process for EnterGame {
    fn process(&mut self) -> Result<(), String> {
        println!("Doing the actual process logic for this connection type!");
        Ok(())
    }
}

///
/// Initial state when entering the game
/// It will send player default entry message
/// and process player input
///
pub struct AwaitingName {
    ptype: PlayerType,
}

impl Process for AwaitingName {
    fn process(&mut self) -> Result<(), String> {
        //
        Ok(())
    }
}

impl From<ConnState<EnterGame>> for ConnState<AwaitingName> {
    fn from(val: ConnState<EnterGame>) -> ConnState<AwaitingName> {
        ConnState {
            client: val.client,
            cmdset: CmdSet::new(),
            state: AwaitingName {
                ptype: PlayerType::NewPlayer("New Player".to_string()),
            },
        }
    }
}

///
/// Asking for password; player exists
/// handle multiple attempts at password.
///
pub struct AwaitingPassword {}

impl From<ConnState<AwaitingName>> for ConnState<AwaitingPassword> {
    fn from(val: ConnState<AwaitingName>) -> ConnState<AwaitingPassword> {
        ConnState {
            client: val.client,
            cmdset: CmdSet::new(),
            state: AwaitingPassword {
                //add fields
            },
        }
    }
}

///
/// New player found, asking for name
///
pub struct AwaitingNewName {}

impl From<ConnState<AwaitingName>> for ConnState<AwaitingNewName> {
    fn from(val: ConnState<AwaitingName>) -> ConnState<AwaitingNewName> {
        ConnState {
            client: val.client,
            cmdset: CmdSet::new(),
            state: AwaitingNewName {
                // add fields
            },
        }
    }
}

///
/// New passwor
///
pub struct AwaitingNewPassword {}

impl From<ConnState<AwaitingNewName>> for ConnState<AwaitingNewPassword> {
    fn from(val: ConnState<AwaitingNewName>) -> ConnState<AwaitingNewPassword> {
        ConnState {
            client: val.client,
            cmdset: CmdSet::new(),
            state: AwaitingNewPassword {
                // add fields
            },
        }
    }
}

///
/// Confirming new password
///
pub struct ConfirmPassword {}

impl From<ConnState<AwaitingNewPassword>> for ConnState<ConfirmPassword> {
    fn from(val: ConnState<AwaitingNewPassword>) -> ConnState<ConfirmPassword> {
        ConnState {
            client: val.client,
            cmdset: CmdSet::new(),
            state: ConfirmPassword {
                // add fields
            },
        }
    }
}

///
/// Main state, player connected to game, available commands.
///
pub struct Playing {}
impl From<ConnState<ConfirmPassword>> for ConnState<Playing> {
    fn from(val: ConnState<ConfirmPassword>) -> ConnState<Playing> {
        ConnState {
            client: val.client,
            cmdset: CmdSet::new(),
            state: Playing {
                // add fields
            },
        }
    }
}

impl From<ConnState<AwaitingPassword>> for ConnState<Playing> {
    fn from(val: ConnState<AwaitingPassword>) -> ConnState<Playing> {
        ConnState {
            client: val.client,
            cmdset: CmdSet::new(),
            state: Playing {
                // add fields
            },
        }
    }
}

///
/// The state right before disconnect
///
pub struct Quiting {}
impl From<ConnState<Playing>> for ConnState<Quiting> {
    fn from(val: ConnState<Playing>) -> ConnState<Quiting> {
        ConnState {
            client: val.client,
            cmdset: CmdSet::new(),
            state: Quiting {
                // add fields
            },
        }
    }
}

pub enum ConnStateDriver {
    EnterGame(ConnState<EnterGame>),
    AwaitingName(ConnState<AwaitingName>),
    AwaitingPassword(ConnState<AwaitingPassword>),
    AwaitingNewName(ConnState<AwaitingNewName>),
    AwaitingNewPassword(ConnState<AwaitingNewPassword>),
    ConfirmPassword(ConnState<ConfirmPassword>),
    Playing(ConnState<Playing>),
    Quiting(ConnState<Quiting>),
}

impl ConnStateDriver {
    pub fn step(mut self) -> Self {
        self = match self {
            ConnStateDriver::EnterGame(val) => ConnStateDriver::AwaitingName(val.into()),
            ConnStateDriver::AwaitingName(val) => match val.state.ptype {
                PlayerType::NewPlayer(_) => ConnStateDriver::AwaitingNewName(val.into()),
                PlayerType::ExistingPlayer(_) => ConnStateDriver::AwaitingPassword(val.into()),
            },
            ConnStateDriver::AwaitingNewName(val) => {
                ConnStateDriver::AwaitingNewPassword(val.into())
            }
            ConnStateDriver::AwaitingNewPassword(val) => {
                ConnStateDriver::ConfirmPassword(val.into())
            }
            ConnStateDriver::ConfirmPassword(val) => ConnStateDriver::Playing(val.into()),
            ConnStateDriver::AwaitingPassword(val) => ConnStateDriver::Playing(val.into()),
            ConnStateDriver::Playing(val) => ConnStateDriver::Quiting(val.into()),
            _ => self,
        };
        self
    }

    ///
    /// Execute main logic for each connstate
    ///
    ///
    pub fn execute(&mut self) {
        println! {"HELLO THERE"}

        if let ConnStateDriver::EnterGame(s) = self {
            s.process().unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
