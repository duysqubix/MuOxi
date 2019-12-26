//!
//! Main Proxy Staging TCP Client communicates with the Game Engine and relays
//! game engine response to connected clients.
//! 

use tokio::io;
use tokio::net::{TcpListener, TcpStream};

use futures::future::try_join;
use futures::FutureExt;

use std::str;
use std::error::Error;


#[tokio::main]
async fn main() -> Result<(), Box< dyn Error>>{
    env_logger::init();

    let listen_addr = "127.0.0.1:8000".to_string();
    let game_addr = "127.0.0.1:4567".to_string();

    println!("TCP Client listening on {} proxying to {}", listen_addr, game_addr);

    let mut listener = TcpListener::bind(&listen_addr).await?;

    while let Ok((inbound, _)) = listener.accept().await{
        // do proxy work here.

        let proxy = transfer(inbound, game_addr.clone()).map(|r|{
            if let Err(e) = r {
                println!("Failed to transfer; error={}", e);
            }
        });
        
        tokio::spawn(proxy);
    }

    Ok(())
}

async fn transfer(mut inbound: TcpStream, game_addr: String) -> Result<(), Box<dyn Error>>{
    let mut outbound = TcpStream::connect(&game_addr).await?;
    let inbound_addr = inbound.peer_addr().unwrap();
    let outbound_addr = outbound.peer_addr().unwrap();


    let mut buf = [0;1024];

    let n = inbound.peek(&mut buf).await?;
    println!("Proxing {} to {}, msg: {}", inbound_addr, outbound_addr, str::from_utf8(&buf[0..n]).unwrap());


    let (mut ri, mut wi) = inbound.split();
    let (mut ro, mut wo) = outbound.split();

    let client_to_server = io::copy(&mut ri, &mut wo);
    let server_to_client = io::copy(&mut ro, &mut wi);


    try_join(client_to_server, server_to_client).await?;
    Ok(())
}
