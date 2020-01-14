#![allow(missing_docs)]
//!
//! Holds the different connection states for connected clients
//!
use crate::comms::Client;
use serde::{Deserialize, Serialize};

/// Different states for connected clients
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ConnStates {
    AwaitingName,
    AwaitingPassword,
    AwaitingNewName,
    AwaitingNewPassword,
    ConfirmNewPassword,
    MainMenu,
    Playing,
    Quit,
}

impl ConnStates {
    /// execute logic based on current connstate, return
    /// a connState to replace original, can even be same one...
    pub fn execute(self, client: &mut Client, response: String) -> Self {
        let response = response.to_lowercase();
        match self {
            ConnStates::AwaitingName => {
                // create cmdset for this state
                ConnStates::AwaitingName
            }
            _ => ConnStates::Quit,
        }
    }
}
