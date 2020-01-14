#![allow(missing_docs)]
//!
//! Holds the different connection states for connected clients
//!
use serde::{Deserialize, Serialize};

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
