#![allow(missing_docs)]

//! Connection states for connected clients.

use crate::cmds::do_cmd;
use crate::cmds::proxy_commands::*;
use crate::cmdset;
use crate::comms::Client;
use crate::prelude::{CmdSet, Command, LinesCodecResult};
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
    /// Validate and execute commands available in the current connection state.
    /// Once a client moves to `Playing`, command availability shifts from
    /// connection-state to in-game state (roles, level, class, etc.).
    pub async fn execute(self, client: &mut Client, response: String) -> LinesCodecResult<Self> {
        match self {
            ConnStates::AwaitingName => {
                let mut cmdset = cmdset![CmdProxyNew, CmdProxyAccount];
                let cmd: Option<&mut (dyn Command + Send)> = cmdset.get(response);
                let errmsg = format!("Error attempting to executing cmd: {:?}", cmd);
                do_cmd(
                    client,
                    cmd,
                    "Login with existing account name using `account [name] or enter `new`",
                )
                .await
                .expect(&errmsg);
                Ok(ConnStates::AwaitingName)
            }
            _ => Ok(ConnStates::Quit),
        }
    }
}
