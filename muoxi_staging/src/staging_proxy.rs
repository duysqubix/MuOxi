//!
//! Main Proxy Staging TCP Client communicates with the Game Engine and relays
//! game engine response to connected clients.
//!

mod copyover;
mod states;

use futures::future::try_join;
use futures::SinkExt;
use states::{AwaitingName, ConnState, EnterGame, MainMenu};
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::{env, str};
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::{Stream, StreamExt};
use tokio::sync::Mutex;
use tokio_util::codec::{Framed, LinesCodec, LinesCodecError};

static GAME_ADDR: &'static str = "127.0.0.1:4567";
static PROXY_ADDR: &'static str = "127.0.0.1:8000";

#[derive(Debug)]
pub enum Message {
    Recieved(String),
}

#[derive(Debug)]
struct Client {
    state: states::ConnState,
    lines: Framed<TcpStream, LinesCodec>,
}

impl Client {
    pub async fn new(stream: TcpStream) -> tokio::io::Result<Self> {
        Ok(Self {
            state: ConnState::AwaitingName(AwaitingName::new()),
            lines: Framed::new(stream, LinesCodec::new()),
        })
    }
}

impl Stream for Client {
    type Item = Result<Message, LinesCodecError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let result: Option<_> = futures::ready!(Pin::new(&mut self.lines).poll_next(cx));

        Poll::Ready(match result {
            Some(Ok(message)) => Some(Ok(Message::Recieved(message))),
            Some(Err(e)) => Some(Err(e)),
            None => None,
        })
    }
}

#[derive(Debug)]
struct Server {
    clients: HashMap<SocketAddr, i32>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    // pub async fn send(&mut self, addr: &SocketAddr, msg: String) -> Result<(), Box<dyn Error>> {
    //     if let Some(c) = self.clients.get_mut(&addr) {
    //         c.lines.send(msg).await?;
    //     }
    //     Ok(())
    // }
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
        let new_client = Arc::clone(&clients);
        println!("New user! on {}", addr);
        tokio::spawn(async move {
            if let Err(e) = process(new_client, inbound).await {
                println!("An error occured; error={:?}", e);
            }
        });
    }

    Ok(())
}

async fn process(server: Arc<Mutex<Server>>, stream: TcpStream) -> Result<(), Box<dyn Error>> {
    // add client to server instance
    let addr = stream.peer_addr()?;
    let mut new_client = Client::new(stream).await?;

    {
        new_client
            .lines
            .send("Please enter `name` or `new`".to_string())
            .await?;
        server.lock().await.clients.insert(addr.clone(), 1);
    }
    // server_send(server, &addr, "Please enter `name` or `new`").await?;

    // loop {
    //     let mut server = server.lock().await;
    //     let client = server.clients.get_mut(&addr);
    // }

    // let mut server = server.lock().await;
    // let client = server.clients.get_mut(&addr);
    // if let Some(c) = client {
    while let Some(result) = new_client.next().await {
        println!("{:?}", result);
    }
    // }

    Ok(())
}

// async fn server_send<'a>(
//     server: Arc<Mutex<Server>>,
//     addr: &SocketAddr,
//     msg: &'a str,
// ) -> Result<(), Box<dyn Error>> {
//     let mut server = server.lock().await;
//     server.send(addr, msg.to_string()).await?;
//     Ok(())
// }

///
/// Example usage
/// ```rust
///     let proxy = transfer(inbound, GAME_ADDR.to_string().clone()).map(|r| {
///        if let Err(e) = r {
///            println!("Failed to transfer; error={}", e);
///        }
///    });
/// ```
///
async fn transfer(mut inbound: TcpStream, game_addr: String) -> Result<(), Box<dyn Error>> {
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

// machine!{
//     enum ConnState{

//     }
// }
// fn main() {}
