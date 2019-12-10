//!
//! File: states.rs
//! Usage: defines all the states a player can be in, and ensures seemless transition between states
//!

///
/// A struct that will allow for sharing between all the states;
///
pub struct SharedData;

enum PlayerType {
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
struct ConnState<T> {
    shared: SharedData,
    cmdset: CmdSet,
    state: T,
}

impl ConnState<AwaitingName> {
    fn new(shared: SharedData, ptype: PlayerType) -> Self {
        ConnState {
            shared: shared,
            cmdset: CmdSet::new(),
            state: AwaitingName { ptype: ptype },
        }
    }
}

///
/// Initial State when entering the game
///
struct AwaitingName {
    ptype: PlayerType,
}

///
/// Asking for password; player exists
///
struct AwaitingPassword {}

impl From<ConnState<AwaitingName>> for ConnState<AwaitingPassword> {
    fn from(val: ConnState<AwaitingName>) -> ConnState<AwaitingPassword> {
        ConnState {
            shared: val.shared,
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
struct AwaitingNewName {}

impl From<ConnState<AwaitingName>> for ConnState<AwaitingNewName> {
    fn from(val: ConnState<AwaitingName>) -> ConnState<AwaitingNewName> {
        ConnState {
            shared: val.shared,
            cmdset: CmdSet::new(),
            state: AwaitingNewName {
                // add fields
            },
        }
    }
}

///
/// New password
///
struct AwaitingNewPassword {}

impl From<ConnState<AwaitingNewName>> for ConnState<AwaitingNewPassword> {
    fn from(val: ConnState<AwaitingNewName>) -> ConnState<AwaitingNewPassword> {
        ConnState {
            shared: val.shared,
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
struct ConfirmPassword {}

impl From<ConnState<AwaitingNewPassword>> for ConnState<ConfirmPassword> {
    fn from(val: ConnState<AwaitingNewPassword>) -> ConnState<ConfirmPassword> {
        ConnState {
            shared: val.shared,
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
struct Playing {}
impl From<ConnState<ConfirmPassword>> for ConnState<Playing> {
    fn from(val: ConnState<ConfirmPassword>) -> ConnState<Playing> {
        ConnState {
            shared: val.shared,
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
            shared: val.shared,
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
struct Quiting {}
impl From<ConnState<Playing>> for ConnState<Quiting> {
    fn from(val: ConnState<Playing>) -> ConnState<Quiting> {
        ConnState {
            shared: val.shared,
            cmdset: CmdSet::new(),
            state: Quiting {
                // add fields
            },
        }
    }
}

enum ConnStateDriver {
    AwaitingName(ConnState<AwaitingName>),
    AwaitingPassword(ConnState<AwaitingPassword>),
    AwaitingNewName(ConnState<AwaitingNewName>),
    AwaitingNewPassword(ConnState<AwaitingNewPassword>),
    ConfirmPassword(ConnState<ConfirmPassword>),
    Playing(ConnState<Playing>),
    Quiting(ConnState<Quiting>),
}

impl ConnStateDriver {
    fn step(mut self) -> Self {
        self = match self {
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
}

#[cfg(test)]
mod test {
    use super::*;
}
