mod comms;

use crate::comms::*;
use futures::{SinkExt, StreamExt};
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_util::codec::{Framed, LinesCodec};

///
/// Main Event Loop of MuOxi. Server listens and accepts new connections
/// and spawns a tokio task that will run concurrently and handle all i/o
/// operations in a thread safe manner.. yay!
///
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let state = Arc::new(Mutex::new(ClientShared::new()));

    let addr = "127.0.0.1:8000".to_string();

    let mut muoxi = TcpListener::bind(&addr).await?;

    println!("MuOxi Server running on: {:?}", muoxi);

    loop {
        let (stream, addr) = muoxi.accept().await?;

        let state = Arc::clone(&state);

        tokio::spawn(async move {
            if let Err(e) = process(state, stream, addr).await {
                println!("An error occured; error={:?}", e);
            }
        });
    }
}

///
/// Wrapper for processing individual client input
///
async fn process(
    state: Arc<Mutex<ClientShared>>,
    stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    let mut lines = Framed::new(stream, LinesCodec::new());
    lines.send("Please enter username".to_string()).await?;

    let username = match lines.next().await {
        Some(Ok(line)) => line,
        _ => {
            println!("Failed to get username from {}", addr);
            return Ok(());
        }
    };

    let mut client = Client::new(state.clone(), lines).await?;

    {
        let mut state = state.lock().await;
        let msg = format!("{} has joined", username);
        println!("{}", msg);
        state.broadcast(addr, &msg).await;
    }

    // process incoming message until our stream is exhausted by a disconnect

    while let Some(result) = client.next().await {
        match result {
            // A message was recieved from the current user, we should
            // do something with this....
            Ok(Message::Broadcast(msg)) => {
                let mut state = state.lock().await;
                let msg = format!("{} say, {}", username, msg);
                state.broadcast(addr, &msg).await;
            }

            // A message was recieved on the RX channel, send it
            // via socket to client
            Ok(Message::Recieved(msg)) => {
                client.lines.send(msg).await?;
            }

            Err(e) => {
                println!(
                    "an error as occured while processing messages for {}; error={:?}",
                    username, e
                );
            }
        }
    }

    {
        let mut state = state.lock().await;
        state.clients.remove(&addr);

        let msg = format!("{} has left the game", username);
        println!("{}", msg);
        state.broadcast(addr, &msg).await;
    }

    Ok(())
}
