#![allow(missing_docs)]
//!
//! Holds the different connection states for connected clients
//!
use crate::cmds::proxy_commands::*;
use crate::cmdset;
use crate::comms::Client;
use crate::prelude::{CmdSet, Command, LinesCodecResult};
use crate::send;
use serde::{Deserialize, Serialize};
use std::marker::Send;

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
    ///
    /// Validates and executes valid commands depending on Connection state
    /// Once client moves to `Playing` state, the list of commands available
    /// will shift from ConnState dependency to game state dependency such as
    /// (roles, level, class, etc..)
    ///
    pub async fn execute(
        self,
        mut client: &mut Client,
        response: String,
    ) -> LinesCodecResult<Self> {
        match self {
            ConnStates::AwaitingName => {
                let mut cmdset = cmdset![CmdProxyNew, CmdProxyAccount];

                // leaving this explicit type for documentation
                // When retrieving a cmd from response, it will return
                // a `&mut (dyn Command + Send)`
                let cmd: Option<&mut (dyn Command + Send)> = cmdset.get(response);
                // retrieve cmd struct based on input
                if let Some(valid_cmd) = cmd {
                    // command is valid continue
                    let msg = format!("Command recognized: {:?}", valid_cmd.name());
                    send(&mut client, &msg).await?;

                    valid_cmd.execute_cmd(&mut client).await.unwrap();
                } else {
                    send(
                        &mut client,
                        "Login with existing account name using `account [name]` or enter `new`",
                    )
                    .await?;
                }
                Ok(ConnStates::AwaitingName)
            }
            _ => Ok(ConnStates::Quit),
        }
    }
}
