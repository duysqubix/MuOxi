#![deny(missing_docs)]

//!
//! ## Main Proxy Staging TCP Server
//!
//! This is where all clients will eventually connect to either via direct connection or
//! one of the supported proxy servers *(MCCP, Webserver, etc..)*
//!
//!

pub mod comms;
pub mod copyover;
pub mod states;

use comms::{Client, Comms, Server};
use db::utils::{gen_uid, UID};
// use db::DatabaseHandler;
use crate::states::ConnStates;
use db::cache_structures::socket::CacheSocket;
use db::cache_structures::Cachable;
use futures::future::try_join;
use futures::SinkExt;
use std::error::Error;
use std::sync::Arc;
use std::{env, str};
use tokio::fs::File;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio::stream::StreamExt;
use tokio::sync::Mutex;
use tokio_util::codec::LinesCodecError;

type LinesCodecResult<T> = Result<T, LinesCodecError>;

/// Current listening port of the MuOxi game engine
pub static GAME_ADDR: &'static str = "127.0.0.1:4567";

/// Current listening port of the staging proxy server
pub static PROXY_ADDR: &'static str = "127.0.0.1:8000";

/// Friendly async wrapper to sending messages to client object
///
/// ```rust
/// let msg = "Hello, and welcome to hell";
/// send(&client, msg).await?;
/// ```
pub async fn send<'a>(client: &'a mut Client, msg: &'a str) -> LinesCodecResult<()> {
    client.lines.send(msg.into()).await?;
    Ok(())
}

/// Friendly async wrapper around recieving message from client
/// Instead of panicing on wrong error, it will return an Option<String>
///
/// ```rust
/// let response = get(&mut client).await;
///
/// if let Some(resp) = response{
///     println!("Recieved from client: {}", resp)
/// }else{
/// println!("Recieved a None, most likely client has prematurely disconnected");
///}
/// ```
pub async fn get<'a>(client: &'a mut Client) -> Option<String> {
    client.lines.next().await.map_or(None, |v| v.ok())
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

    // remove client from database
    if let Ok(_) = cache.destruct() {
        println!("Remove client with uid: {}", uid);
    } else {
        println!("Unable to remove client: {} from redis.", uid);
    }
}

///
/// Main processing piece of logic, once a connection is established to client
/// the entire lifetime of the connected client is handled within this function.
///
pub async fn process(
    server: Arc<Mutex<Server>>,
    stream: TcpStream,
    mut cache: CacheSocket,
) -> Result<(), Box<dyn Error>> {
    // could use refactoring, but UID is important that all structures
    // have know about this UID in Server, Client, and CacheSocket
    // So it attempts to retrieve UID from redis, and use that as the
    // primary source for UID, if there is an error, it will attempt to
    // reassign UID and store to redis - this may be redundant, I am not
    // quite sure how I should handle if there is an error retrieving
    // a uid
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

    // create client connec
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
            // here is where we process input based on connection state
            let resp = format!("You said, {}", response);
            send(&mut client, &resp).await?;

        // for testing use server to use id
        // valid reponse
        } else {
            // most likely disconnected, cleanup
            println!("Client dropped connection. Removing...");
            game_loop = false;
        }
    }

    // let mut new_process = true;
    // let mut acct_error_counter: usize = 0;
    // while new_process {
    //     let response = get(&mut client).await;
    //     println!("Im here");
    //     match response.to_lowercase().as_str() {
    //         "new" => {
    //             let greetings = format!(
    //                 "{}\r\n{}\r\n",
    //                 "Puff the Magic Dragon says, Welcome to MuOxi, enjoy your stay. :)",
    //                 "What be your name?"
    //             );
    //             send(&mut client, greetings.as_str()).await?;
    //             let new_name = get(&mut client).await;

    //             // {
    //             //     let server = server.lock().await;
    //             // }

    //             let r = format!("Welcome, {}! Glad to have you on board.", new_name);

    //             send(&mut client, r.as_str()).await?;
    //             new_process = false;
    //             continue;
    //         }
    //         _ => {
    //             if acct_error_counter == 3 {
    //                 send(&mut client, "Max attempts reached.. disconnecting\r\n").await?;
    //                 new_process = false;
    //                 continue;
    //             }
    //             // println!("Attempting to find client...");
    //             let err_msg = format!("Couldn't find account name with `{}`\r\n", response);
    //             send(&mut client, err_msg.as_str()).await?;
    //             acct_error_counter += 1;
    //         }
    //     }
    // }

    // let name = client.lines.next().await;
    // if let Some(v) = name {
    //     if let Ok(x) = v {
    //         client.name = x.clone();
    //     }
    // }

    // process clients input until a disconnect happens
    // while let Some(result) = client.next().await {
    //     match result {
    //         // Information coming in from individual client
    //         Ok(Message::FromClient(msg)) => {
    //             let resp = format!("You say, {}", msg);
    //             client.lines.send(resp).await?;
    //             {
    //                 let mut server = server.lock().await;
    //                 let msg = format!("{} says, {}", client.name, msg);
    //                 server.broadcast(addr, &msg).await;
    //             }
    //         }

    //         // Information coming in from Clients Rx channel
    //         Ok(Message::OnRx(msg)) => {
    //             // process information coming from other clients
    //             client.lines.send(msg).await?;
    //         }
    //         // An error has occured
    //         Err(e) => {
    //             println!(
    //                 "an error occured whiel processing input for {}; error={:?}",
    //                 &addr, e
    //             );
    //         }
    //     }
    // }
    // user disconnecting, remove from server list
    client_cleanup(uid, &server, cache).await;
    Ok(())
}

/// Turns the staging server into a full proxy server, relaying information sent
/// to proxy/staging server to MuOxi game engine
///
/// ### Example usage
/// ```rust
///     let _proxy = transfer(inbound, GAME_ADDR.to_string().clone()).map(|r| {
///        if let Err(e) = r {
///            println!("Failed to transfer; error={}", e);
///        }
///    });
/// ```
///
pub async fn transfer(mut inbound: TcpStream, game_addr: String) -> Result<(), Box<dyn Error>> {
    let mut outbound = TcpStream::connect(&game_addr).await?;
    let inbound_addr = inbound.peer_addr()?;
    let outbound_addr = outbound.peer_addr()?;

    let mut buf = [0; 1024];

    let n = inbound.peek(&mut buf).await?;
    println!(
        "Proxing {} to {}, msg: {}",
        inbound_addr,
        outbound_addr,
        str::from_utf8(&buf[0..n])?
    );

    let (mut ri, mut wi) = inbound.split();
    let (mut ro, mut wo) = outbound.split();

    let client_to_server = copyover::copy(&mut ri, &mut wo, &inbound_addr, &outbound_addr);
    let server_to_client = copyover::copy(&mut ro, &mut wi, &outbound_addr, &inbound_addr);

    try_join(client_to_server, server_to_client).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env::set_var("RUST_LOG", "info, warn, error,test");
    pretty_env_logger::init();

    let clients = Arc::new(Mutex::new(Server::new()));

    println!(
        "TCP Client listening on {} proxying to {}",
        PROXY_ADDR, GAME_ADDR
    );

    let mut listener = TcpListener::bind(&PROXY_ADDR).await?;

    while let Ok((stream, addr)) = listener.accept().await {
        // For each inbound client - step through states and only when
        // when entering game does it invoke the transfer async function
        let server = Arc::clone(&clients);
        println!("New user! on {}", addr);

        // obtain address of client
        let addr = stream.peer_addr()?;

        // stores client information in redis and dynamically creates a UID
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
