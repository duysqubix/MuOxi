#![allow(missing_docs)]

//! Connection-state machine.

use crate::cmds::dispatch;
use crate::comms::Client;
use crate::prelude::LinesCodecResult;
use crate::registry::Registry;
use crate::world::WorldApi;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
    /// Drive the state machine one step. Plan 6 fills in all variants; for
    /// this plan, only the previously-existing `AwaitingName` arm and the
    /// new `Playing` arm are functional.
    pub async fn execute(
        self,
        client: &mut Client,
        registry: Arc<Registry>,
        world: Arc<WorldApi>,
        response: String,
    ) -> LinesCodecResult<Self> {
        match self {
            ConnStates::AwaitingName => {
                if response.eq_ignore_ascii_case("new") {
                    Ok(ConnStates::AwaitingNewName)
                } else if !response.trim().is_empty() {
                    Ok(ConnStates::AwaitingPassword)
                } else {
                    Ok(ConnStates::AwaitingName)
                }
            }
            ConnStates::Playing => {
                if response.trim().eq_ignore_ascii_case("quit") {
                    return Ok(ConnStates::Quit);
                }
                dispatch(client, registry, world, &response, "Huh?").await;
                Ok(ConnStates::Playing)
            }
            _ => Ok(ConnStates::Quit),
        }
    }
}
