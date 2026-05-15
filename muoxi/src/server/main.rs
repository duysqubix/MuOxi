#![deny(missing_docs)]

//! ## MuOxi Server (unified staging + engine binary)
//!
//! Single Tokio runtime hosting protocol/login state machine and in-process
//! game logic. Clients connect via direct TCP at `PROXY_ADDR` (default
//! `127.0.0.1:8000`) or via the `muoxi_web` WebSocket bridge. When a session
//! enters [`states::ConnStates::Playing`], per-line input is dispatched into
//! [`engine::handle_input`].
//!
//! The v0.2 roadmap reintroduces a portal/server boundary with a framed
//! protocol; until then this is one process.

pub mod cmds;
pub mod comms;
pub mod engine;
pub mod prelude;
pub mod states;
pub mod world;

use crate::prelude::LinesCodecResult;
use crate::states::ConnStates;
use comms::{Client, Server};
use db::cache_structures::Cachable;
use db::cache_structures::socket::CacheSocket;
use db::utils::{UID, gen_uid};
use futures_util::SinkExt;
use std::error::Error;
use std::sync::Arc;
use std::{env, str};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_stream::StreamExt;

/// Friendly async wrapper for sending messages to a client.
pub async fn send<'a>(client: &'a mut Client, msg: &'a str) -> LinesCodecResult<()> {
    client.lines.send(msg.to_string()).await?;
    Ok(())
}

/// Friendly async wrapper around recieving message from client.
/// Instead of panicing on wrong error, it returns an `Option<String>`.
pub async fn get<'a>(client: &'a mut Client) -> Option<String> {
    client.lines.next().await.and_then(|v| v.ok())
}

/// display welcome screen
pub async fn display_welcome<'a>(client: &'a mut Client) -> LinesCodecResult<()> {
    let mut file = File::open("resources/welcome.txt").await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    client.lines.send(contents).await?;
    Ok(())
}

/// clean up client on disconnect or timeout
pub async fn client_cleanup(uid: UID, server: &Arc<Mutex<Server>>, cache: CacheSocket) {
    let mut server = server.lock().await;
    server.clients.remove(&uid);

    if cache.destruct().is_ok() {
        println!("Remove client with uid: {}", uid);
    } else {
        println!("Unable to remove client: {} from redis.", uid);
    }
}

/// Main per-client processing loop. The entire lifetime of the connected
/// client is handled within this function.
pub async fn process(
    server: Arc<Mutex<Server>>,
    stream: TcpStream,
    mut cache: CacheSocket,
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
    client.state = ConnStates::AwaitingName;

    display_welcome(&mut client).await?;
    let mut game_loop = true;
    while game_loop {
        if client.state == ConnStates::Quit {
            println!("Client is disconnecting");
            game_loop = false;
        }
        if let Some(response) = get(&mut client).await {
            let new_state = client.state.clone().execute(&mut client, response).await?;
            client.state = new_state;
            let state = format!("({:?})", client.state);
            send(&mut client, &state).await?;
        } else {
            println!("Client dropped connection. Removing...");
            game_loop = false;
        }
    }

    client_cleanup(uid, &server, cache).await;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    unsafe {
        env::set_var("RUST_LOG", "info,warn,error,test");
    }
    let proxy_addr: String =
        env::var("PROXY_ADDR").unwrap_or_else(|_| "127.0.0.1:8000".to_string());

    pretty_env_logger::init();

    let clients = Arc::new(Mutex::new(Server::new()));

    println!("MuOxi server listening on {}", proxy_addr);

    let listener = TcpListener::bind(&proxy_addr).await?;

    while let Ok((stream, addr)) = listener.accept().await {
        let server = Arc::clone(&clients);
        println!("New user! on {}", addr);

        let addr = stream.peer_addr()?;

        let mut cache_socket = CacheSocket::new();
        cache_socket.set_address(&addr).dump()?;

        tokio::spawn(async move {
            if let Err(e) = process(server, stream, cache_socket).await {
                println!("An error occured; error={:?}", e);
            }
        });
    }

    Ok(())
}
