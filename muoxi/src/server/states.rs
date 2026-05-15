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
            ConnStates::AwaitingNewName => {
                let trimmed = response.trim();
                if !crate::auth::is_valid_name(trimmed) {
                    crate::send(
                        client,
                        "Names are 3-32 chars, alphanumeric/underscore, and start with a letter. Try again:",
                    )
                    .await?;
                    return Ok(ConnStates::AwaitingNewName);
                }
                if world.find_account_by_name(trimmed).await.is_some() {
                    crate::send(client, "That name is taken. Choose another:").await?;
                    return Ok(ConnStates::AwaitingNewName);
                }
                client.auth_buffer.pending_name = Some(trimmed.to_string());
                crate::send(client, "Password (6+ chars, no whitespace):").await?;
                Ok(ConnStates::AwaitingNewPassword)
            }
            ConnStates::AwaitingNewPassword => {
                if !crate::auth::is_valid_password(&response) {
                    crate::send(
                        client,
                        "Password must be 6+ chars with no whitespace. Try again:",
                    )
                    .await?;
                    return Ok(ConnStates::AwaitingNewPassword);
                }
                client.auth_buffer.first_password_attempt = Some(response.clone());
                crate::send(client, "Confirm password:").await?;
                Ok(ConnStates::ConfirmNewPassword)
            }
            ConnStates::ConfirmNewPassword => {
                let first = client.auth_buffer.first_password_attempt.clone();
                let name = client.auth_buffer.pending_name.clone();
                let (Some(first), Some(name)) = (first, name) else {
                    client.auth_buffer.clear();
                    crate::send(client, "Session lost. Enter your account name:").await?;
                    return Ok(ConnStates::AwaitingName);
                };
                if first != response {
                    client.auth_buffer.first_password_attempt = None;
                    crate::send(client, "Passwords don't match. Enter password again:").await?;
                    return Ok(ConnStates::AwaitingNewPassword);
                }
                let hash = match crate::auth::hash_password(&response) {
                    Ok(h) => h,
                    Err(e) => {
                        crate::send(client, &format!("Internal error: {e}. Disconnecting."))
                            .await?;
                        return Ok(ConnStates::Quit);
                    }
                };
                let acct = match world.create_account(&name, &hash, "").await {
                    Ok(a) => a,
                    Err(e) => {
                        client.auth_buffer.clear();
                        crate::send(client, &format!("Could not create account: {e}.")).await?;
                        return Ok(ConnStates::AwaitingName);
                    }
                };
                client.account_uid = Some(acct.uid);
                client.auth_buffer.clear();
                crate::send(client, &format!("Account {} created.", acct.name)).await?;
                crate::send(
                    client,
                    "Type `new <name>` to create your first character, or `quit`.",
                )
                .await?;
                Ok(ConnStates::MainMenu)
            }
            ConnStates::MainMenu => {
                let Some(account_uid) = client.account_uid else {
                    client.auth_buffer.clear();
                    crate::send(client, "Session lost. Enter your account name:").await?;
                    return Ok(ConnStates::AwaitingName);
                };
                let trimmed = response.trim();
                let chars = world.list_account_characters(account_uid).await;

                if trimmed.is_empty() {
                    let mut menu = String::new();
                    if chars.is_empty() {
                        menu.push_str("You have no characters yet.\n");
                    } else {
                        menu.push_str("Your characters:\n");
                        for (idx, ch) in chars.iter().enumerate() {
                            menu.push_str(&format!("  {}. {}\n", idx + 1, ch.name));
                        }
                    }
                    menu.push_str("Enter a number to play, `new <name>` to create, or `quit`.");
                    crate::send(client, &menu).await?;
                    return Ok(ConnStates::MainMenu);
                }

                if trimmed.eq_ignore_ascii_case("quit") {
                    return Ok(ConnStates::Quit);
                }

                let create_arg = if let Some(rest) = trimmed.strip_prefix("new ") {
                    Some(rest.trim())
                } else if trimmed.eq_ignore_ascii_case("new") {
                    Some("")
                } else {
                    None
                };
                if let Some(name) = create_arg {
                    if !crate::auth::is_valid_name(name) {
                        crate::send(
                            client,
                            "Usage: `new <name>` — 3-32 chars, alphanumeric/underscore, must start with a letter.",
                        )
                        .await?;
                        return Ok(ConnStates::MainMenu);
                    }
                    let starting_room = world.starting_room().await;
                    match world
                        .create_character(&registry, account_uid, name, starting_room)
                        .await
                    {
                        Ok(obj) => {
                            client.character_uid = Some(obj.uid);
                            crate::send(
                                client,
                                &format!("Created {}. Entering world.", obj.name),
                            )
                            .await?;
                            return Ok(ConnStates::Playing);
                        }
                        Err(e) => {
                            crate::send(client, &format!("Could not create: {e}")).await?;
                            return Ok(ConnStates::MainMenu);
                        }
                    }
                }

                if let Ok(idx) = trimmed.parse::<usize>() {
                    if idx >= 1 && idx <= chars.len() {
                        let chosen = &chars[idx - 1];
                        client.character_uid = Some(chosen.uid);
                        crate::send(client, &format!("Playing as {}.", chosen.name))
                            .await?;
                        return Ok(ConnStates::Playing);
                    }
                }

                crate::send(
                    client,
                    "Unrecognized. Enter a number, `new <name>`, or `quit`.",
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
