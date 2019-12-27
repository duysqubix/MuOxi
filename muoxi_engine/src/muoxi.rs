//!
//! The main MuOxi Game server. This is where all the game
//! logic exists and interaction to this server will be on
//! port: 4567
//! 
use std::str;
use tokio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{

    let game_server_addr = "127.0.0.1:4567".to_string();

    let mut game_server = TcpListener::bind(&game_server_addr).await?;

    println!("MuOxi Game Engine listening on: {}", game_server_addr);

    loop{
        let (mut socket, _) = game_server.accept().await?;

        let process = async move{
            let mut buf = [0;1024];

            loop{
                let n = socket.read(&mut buf).await.expect("Failed to read from socket");
                
                if n == 0 {
                    return;
                }
                let msg = str::from_utf8(&buf[0..n]);

                if let Ok(msg) = msg{
                    let resp = format!("Game Server > {}", msg);
                    socket.write_all(resp.as_bytes())
                    .await
                    .expect("Failed to write data to socket");
                }
                else{
                    println!("Unable to decode UTF-8 string.");
                }
            }
        };

        tokio::spawn(process);
    }
}

