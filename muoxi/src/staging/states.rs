#![allow(missing_docs)]
//!
//! Holds the different connection states for connected clients
//!
use crate::cmds::proxy_commands::*;
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
    /// execute logic based on current connstate, return
    /// a connState to replace original, can even be same one...
    pub async fn execute(
        self,
        mut client: &mut Client,
        response: String,
    ) -> LinesCodecResult<Self> {
        match self {
            ConnStates::AwaitingName => {
                // construct valid commands for this state
                let mut cmdset =
                    CmdSet::new(vec![Box::new(CmdProxyNew), Box::new(CmdProxyAccount)]);

                let cmd: Option<&mut (dyn Command + Send)> = cmdset.get(response);
                // retrieve cmd struct based on input
                if let Some(valid_cmd) = cmd {
                    // command is valid continue
                    let msg = format!("Command recognized: {:?}", valid_cmd.name());
                    send(&mut client, &msg).await?;

                    valid_cmd.execute_cmd(&mut client).await.unwrap();
                } else {
                    send(&mut client, "Huh?").await?;
                }
                Ok(ConnStates::AwaitingName)
            }
            _ => Ok(ConnStates::Quit),
        }
    }
}
