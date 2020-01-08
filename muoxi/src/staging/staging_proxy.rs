//!
//! ## Main Proxy Staging TCP Server
//!
//! This is where all clients will eventually connect to either via direct connection or
//! one of the supported proxy servers *(MCCP, Webserver, etc..)*
//!
//!

pub mod comms;
pub mod copyover;

use comms::{Client, ClientAccount, Comms, Server};
use db::utils::{gen_uid, UID};
use futures::future::try_join;
use futures::Future;
use futures::SinkExt;
use rand::prelude::*;
use states::ConnStates;
use std::error::Error;
use std::sync::Arc;
use std::{env, str};
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::{Stream, StreamExt};
use tokio::sync::Mutex;
use tokio_util::codec::LinesCodecError;

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
pub async fn send<'a>(client: &'a mut Client, msg: &'a str) -> Result<(), LinesCodecError> {
    client.lines.send(msg.into()).await?;
    Ok(())
}

/// Friendly async wrapper around recieving message from client
/// Instead of panicing on wrong error, it will return a client disconnect message
///
/// ```rust
/// let client_response = get(&client).await;
/// println!("Recieved from client: {}", client_response);
/// ```
pub async fn get<'a>(client: &'a mut Client) -> String {
    if let Some(Ok(v)) = client.lines.next().await {
        v
    } else {
        "...Client Disconnected...".to_string()
    }
}

///
/// Main processing piece of logic, once a connection is established to client
/// the entire lifetime of the connected client is handled within this function.
///
pub async fn process(server: Arc<Mutex<Server>>, stream: TcpStream) -> Result<(), Box<dyn Error>> {
    // add client to server instance
    // let uid: UID = rand::thread_rng().gen();
    let uid = gen_uid();
    let addr = stream.peer_addr()?;
    let mut new_client = Client::new(uid.clone(), server.clone(), stream).await?;

    // i

    // send intro message
    // name -> try to find account
    // new -> create new account
    let mut new_process = true;
    let mut acct_error_counter: usize = 0;
    while new_process {
        send(&mut new_client, "Please enter account `name` or type `new`").await?;
        let response = get(&mut new_client).await;
        match response.to_lowercase().as_str() {
            "new" => {
                let greetings = format!(
                    "{}\r\n{}\r\n",
                    "Puff the Magic Dragon says, Welcome to MuOxi, enjoy your stay. :)",
                    "What be your name?"
                );
                send(&mut new_client, greetings.as_str()).await?;
                let new_name = get(&mut new_client).await;

                new_client.state = ConnStates::AwaitingName;
                let new_acct = ClientAccount::new(new_name.clone());

                {
                    let mut server = server.lock().await;
                    server.accounts.insert(uid.clone(), new_acct);
                }

                let r = format!("Welcome, {}! Glad to have you on board.", new_name);

                send(&mut new_client, r.as_str()).await?;
                new_process = false;
                continue;
            }
            _ => {
                if acct_error_counter == 3 {
                    send(&mut new_client, "Max attempts reached.. disconnecting\r\n").await?;
                    new_process = false;
                    continue;
                }
                // println!("Attempting to find client...");
                let err_msg = format!("Couldn't find account name with `{}`\r\n", response);
                send(&mut new_client, err_msg.as_str()).await?;
                acct_error_counter += 1;
            }
        }
    }

    {
        let server = server.lock().await;

        for (uid, acct) in server.accounts.iter() {
            println!("Name: {}", acct.name);
            if let Some(comms) = server.clients.get(&uid.clone()) {
                let Comms(socket, _) = comms;
                let msg = format!(
                    "Hello, {}. You belong to port: {}/ {}",
                    acct.name, socket, uid
                );
                send(&mut new_client, msg.as_str()).await?;
            }
        }
    }

    // let name = new_client.lines.next().await;
    // if let Some(v) = name {
    //     if let Ok(x) = v {
    //         new_client.name = x.clone();
    //     }
    // }

    // process clients input until a disconnect happens
    // while let Some(result) = new_client.next().await {
    //     match result {
    //         // Information coming in from individual client
    //         Ok(Message::FromClient(msg)) => {
    //             let resp = format!("You say, {}", msg);
    //             new_client.lines.send(resp).await?;
    //             {
    //                 let mut server = server.lock().await;
    //                 let msg = format!("{} says, {}", new_client.name, msg);
    //                 server.broadcast(addr, &msg).await;
    //             }
    //         }

    //         // Information coming in from Clients Rx channel
    //         Ok(Message::OnRx(msg)) => {
    //             // process information coming from other clients
    //             new_client.lines.send(msg).await?;
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
    {
        let mut server = server.lock().await;
        server.clients.remove(&uid);
        server.accounts.remove(&uid);

        let msg = format!("{} has disconnected", &addr);
        server.broadcast(addr, &msg).await;
    }

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
    let inbound_addr = inbound.peer_addr().unwrap();
    let outbound_addr = outbound.peer_addr().unwrap();

    let mut buf = [0; 1024];

    let n = inbound.peek(&mut buf).await?;
    println!(
        "Proxing {} to {}, msg: {}",
        inbound_addr,
        outbound_addr,
        str::from_utf8(&buf[0..n]).unwrap()
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

    while let Ok((inbound, addr)) = listener.accept().await {
        // For each inbound client - step through states and only when
        // when entering game does it invoke the transfer async function
        let server = Arc::clone(&clients);
        println!("New user! on {}", addr);
        tokio::spawn(async move {
            if let Err(e) = process(server, inbound).await {
                println!("An error occured; error={:?}", e);
            }
        });
    }

    Ok(())
}
