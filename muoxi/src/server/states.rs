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
                let trimmed = response.trim();
                if trimmed.eq_ignore_ascii_case("new") {
                    crate::send(
                        client,
                        "Choose an account name (3-32 chars, alphanumeric, start with letter):",
                    )
                    .await?;
                    return Ok(ConnStates::AwaitingNewName);
                }
                if trimmed.is_empty() {
                    crate::send(
                        client,
                        "Enter your account name (or `new` to create one):",
                    )
                    .await?;
                    return Ok(ConnStates::AwaitingName);
                }
                if !crate::auth::is_valid_name(trimmed) {
                    crate::send(
                        client,
                        "Invalid name. Type `new` to create an account, or enter your existing account name:",
                    )
                    .await?;
                    return Ok(ConnStates::AwaitingName);
                }
                match world.find_account_by_name(trimmed).await {
                    Some(account) => {
                        client.auth_buffer.pending_name = Some(account.name.clone());
                        client.account_uid = Some(account.uid);
                        crate::send(client, "Password:").await?;
                        Ok(ConnStates::AwaitingPassword)
                    }
                    None => {
                        crate::send(
                            client,
                            &format!(
                                "No account named {:?}. Enter another name, or `new`:",
                                trimmed
                            ),
                        )
                        .await?;
                        Ok(ConnStates::AwaitingName)
                    }
                }
            }
            ConnStates::AwaitingPassword => {
                let Some(account_uid) = client.account_uid else {
                    client.auth_buffer.clear();
                    crate::send(client, "Session lost. Enter your account name:").await?;
                    return Ok(ConnStates::AwaitingName);
                };
                let Some(name) = client.auth_buffer.pending_name.clone() else {
                    client.auth_buffer.clear();
                    client.account_uid = None;
                    crate::send(client, "Session lost. Enter your account name:").await?;
                    return Ok(ConnStates::AwaitingName);
                };
                let stored_hash = world.account_password_hash(account_uid).await;
                let ok = match stored_hash {
                    Some(h) => crate::auth::verify_password(&response, &h),
                    None => false,
                };
                if !ok {
                    client.auth_buffer.clear();
                    client.account_uid = None;
                    crate::send(client, "Bad password. Enter your account name:").await?;
                    return Ok(ConnStates::AwaitingName);
                }
                client.auth_buffer.clear();
                crate::send(client, &format!("Welcome, {}.", name)).await?;

                let world_ref: &WorldApi = world.as_ref();
                let session_uid = client.uid;
                registry
                    .hooks
                    .emit(|h| async move {
                        let mut ctx = crate::hooks::HookContext {
                            world: world_ref,
                            session_uid: Some(session_uid),
                        };
                        h.at_login(&mut ctx, account_uid).await
                    })
                    .await;

                crate::send(
                    client,
                    "(press Enter for your character list, or type `new <name>` / `quit`)",
                )
                .await?;
                Ok(ConnStates::MainMenu)
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
