//! MuOxi MUD framework — extension surface.
//!
//! Downstream MUDs depend on this crate to gain access to the framework's
//! types (`Registry`, `Command`, `Hook`, `TypeClass`, `WorldApi`) and the
//! built-in commands / typeclasses. The provided `muoxi_server` binary is
//! one instance of an embedding — downstream developers vendor or fork the
//! `main()` body to inject their own type/command/hook registrations before
//! the listener spawns.
//!
//! See [`registry::Registry`] for the central extension surface.

#[path = "server/auth.rs"]
pub mod auth;
#[path = "server/cmds.rs"]
pub mod cmds;
#[path = "server/commands/mod.rs"]
pub mod commands;
#[path = "server/comms.rs"]
pub mod comms;
#[path = "server/engine.rs"]
pub mod engine;
#[path = "server/hooks.rs"]
pub mod hooks;
#[path = "server/locks.rs"]
pub mod locks;
#[path = "server/prelude.rs"]
pub mod prelude;
#[path = "server/registry.rs"]
pub mod registry;
#[path = "server/scheduler.rs"]
pub mod scheduler;
#[path = "server/scripts/mod.rs"]
pub mod scripts;
#[path = "server/seed.rs"]
pub mod seed;
#[path = "server/states.rs"]
pub mod states;
#[path = "server/typeclass.rs"]
pub mod typeclass;
#[path = "server/world.rs"]
pub mod world;

use crate::comms::{Client, Message, Server};
use crate::prelude::LinesCodecResult;
use crate::registry::Registry;
use crate::states::ConnStates;
use crate::world::WorldApi;
use db::cache_structures::Cachable;
use db::cache_structures::socket::CacheSocket;
use db::utils::{UID, gen_uid};
use futures_util::SinkExt;
use std::error::Error;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;

/// Per-session config that the listener loop passes to each spawned `process()`.
#[derive(Clone, Copy, Debug)]
pub struct SessionConfig {
    /// If `Some(room_uid)`, skip the auth state machine on connect: create a
    /// throwaway "Dev" character placed in `room_uid`, set `Client::uid` to
    /// that character's UID, and jump straight to `ConnStates::Playing`. Set
    /// by the `DEV_AUTOLOGIN` env var at server startup.
    pub dev_autologin_room: Option<UID>,
}

/// Friendly async wrapper for sending messages to a client.
pub async fn send(client: &mut Client, msg: &str) -> LinesCodecResult<()> {
    client.lines.send(msg.to_string()).await?;
    Ok(())
}

/// Friendly async wrapper around recieving message from client.
/// Instead of panicing on wrong error, it returns an `Option<String>`.
pub async fn get(client: &mut Client) -> Option<String> {
    client.lines.next().await.and_then(|v| v.ok())
}

/// Send the welcome banner from `resources/welcome.txt` (relative to CWD).
pub async fn display_welcome(client: &mut Client) -> LinesCodecResult<()> {
    let mut file = File::open("resources/welcome.txt").await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    client.lines.send(contents).await?;
    Ok(())
}

/// Remove the client from the server roster, clear its Redis socket entry,
/// and fire `at_disconnect` for every registered hook (when `account_uid`
/// is `Some`, i.e. the session had completed login).
pub async fn client_cleanup(
    uid: UID,
    account_uid: Option<UID>,
    server: &Arc<Mutex<Server>>,
    registry: Arc<Registry>,
    cache: CacheSocket,
) {
    {
        let mut server = server.lock().await;
        server.clients.remove(&uid);
    }
    if cache.destruct().is_ok() {
        println!("Removed client uid: {}", uid);
    } else {
        println!("Unable to remove client {} from redis.", uid);
    }
    if let Some(acc) = account_uid {
        let world = registry.world.clone();
        let world_ref: &WorldApi = world.as_ref();
        registry
            .hooks
            .emit(|h| async move {
                let mut ctx = crate::hooks::HookContext {
                    world: world_ref,
                    session_uid: Some(uid),
                };
                h.at_disconnect(&mut ctx, acc).await
            })
            .await;
    }
}

/// Per-client processing loop. The entire lifetime of the connected
/// client is handled within this function.
pub async fn process(
    server: Arc<Mutex<Server>>,
    registry: Arc<Registry>,
    world: Arc<WorldApi>,
    stream: TcpStream,
    mut cache: CacheSocket,
    config: SessionConfig,
) -> Result<(), Box<dyn Error>> {
    let uid = cache.get_value::<UID>("uid").unwrap_or_else(|| {
        println!("Error retrieving UID from redis, reassigning UID");
        let new_uid = gen_uid();
        if let Err(e) = cache.set_value("uid", new_uid) {
            println!(
                "{}\nUnable to set key/value pair in redis uid: {}",
                e, new_uid
            );
        };
        new_uid
    });

    let mut client = Client::new(uid, server.clone(), stream).await?;

    if let Some(starting_room) = config.dev_autologin_room {
        match world
            .create_object("character", "Dev", Some(starting_room))
            .await
        {
            Ok(dev_char) => {
                client.character_uid = Some(dev_char.uid);
                {
                    let mut srv = server.lock().await;
                    if let Some(comms) = srv.clients.get_mut(&client.uid) {
                        comms.character_uid = Some(dev_char.uid);
                    }
                }
                client.state = ConnStates::Playing;
                let _ = send(
                    &mut client,
                    "[DEV AUTOLOGIN] You are 'Dev'. Try: look, say hello, who, quit.",
                )
                .await;
            }
            Err(e) => {
                eprintln!(
                    "DEV_AUTOLOGIN failed to create dev character: {e}; falling back to auth flow"
                );
                client.state = ConnStates::AwaitingName;
                display_welcome(&mut client).await?;
            }
        }
    } else {
        client.state = ConnStates::AwaitingName;
        display_welcome(&mut client).await?;
    }

    let mut game_loop = true;
    while game_loop {
        if client.state == ConnStates::Quit {
            println!("Client is disconnecting");
            game_loop = false;
            continue;
        }
        match client.next().await {
            Some(Ok(Message::FromClient(response))) => {
                let new_state = client
                    .state
                    .clone()
                    .execute(
                        &mut client,
                        registry.clone(),
                        world.clone(),
                        server.clone(),
                        response,
                    )
                    .await?;
                client.state = new_state;
                let state = format!("({:?})", client.state);
                send(&mut client, &state).await?;
            }
            Some(Ok(Message::OnRx(broadcast))) => {
                send(&mut client, &broadcast).await?;
            }
            Some(Err(_)) | None => {
                println!("Client dropped connection. Removing...");
                game_loop = false;
            }
        }
    }

    client_cleanup(uid, client.account_uid, &server, registry, cache).await;
    Ok(())
}
