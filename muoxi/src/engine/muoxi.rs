//! The main MuOxi Game server. All game logic lives here. Listens on
//! `127.0.0.1:4567` (override with `GAME_ADDR` env var if needed in the
//! future). Currently a simple echo server while the engine design settles.

use std::env;
use std::error::Error;
use std::str;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let game_server_addr =
        env::var("GAME_ADDR").unwrap_or_else(|_| "127.0.0.1:4567".to_string());

    let game_server = TcpListener::bind(&game_server_addr).await?;

    println!("MuOxi Game Engine listening on: {}", game_server_addr);

    loop {
        let (mut socket, _) = game_server.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(0) => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("Failed to read from socket: {}", e);
                        return;
                    }
                };

                let msg = match str::from_utf8(&buf[..n]) {
                    Ok(m) => m,
                    Err(_) => {
                        eprintln!("Unable to decode UTF-8 string.");
                        continue;
                    }
                };

                let resp = format!("Game Server > {}", msg);
                if let Err(e) = socket.write_all(resp.as_bytes()).await {
                    eprintln!("Failed to write data to socket: {}", e);
                    return;
                }
            }
        });
    }
}
