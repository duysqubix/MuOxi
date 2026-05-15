//! WebSocket-to-TCP bridge. Listens on `127.0.0.1:8080` for browser clients
//! and, for each WebSocket connection, opens a fresh outbound TCP connection
//! to the staging proxy at `127.0.0.1:8000`. Text messages are forwarded both
//! ways (line-oriented).
//!
//! Run after starting `muoxi_staging`.
//!
//! ```bash
//! cargo run --bin muoxi_staging   # in one terminal
//! cargo run --bin muoxi_web       # in another
//! # then connect with any WS client to ws://127.0.0.1:8080
//! ```

use futures_util::{SinkExt, StreamExt};
use std::env;
use std::error::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

type AnyError = Box<dyn Error + Send + Sync>;

async fn handle_client(ws_stream: TcpStream, staging_addr: String) -> Result<(), AnyError> {
    let ws = accept_async(ws_stream).await?;
    let (mut ws_tx, mut ws_rx) = ws.split();

    let tcp = TcpStream::connect(&staging_addr).await?;
    let (tcp_r, mut tcp_w) = tcp.into_split();

    let ws_to_tcp = tokio::spawn(async move {
        while let Some(msg) = ws_rx.next().await {
            match msg? {
                Message::Text(text) => {
                    let line = format!("{}\n", text);
                    tcp_w.write_all(line.as_bytes()).await?;
                }
                Message::Binary(bytes) => {
                    tcp_w.write_all(&bytes).await?;
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
        let _ = tcp_w.shutdown().await;
        Ok::<_, AnyError>(())
    });

    let tcp_to_ws = tokio::spawn(async move {
        let mut lines = BufReader::new(tcp_r).lines();
        while let Some(line) = lines.next_line().await? {
            ws_tx.send(Message::Text(line)).await?;
        }
        let _ = ws_tx.send(Message::Close(None)).await;
        Ok::<_, AnyError>(())
    });

    let _ = tokio::join!(ws_to_tcp, tcp_to_ws);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), AnyError> {
    pretty_env_logger::init();

    let listen_addr =
        env::var("WEB_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let staging_addr =
        env::var("PROXY_ADDR").unwrap_or_else(|_| "127.0.0.1:8000".to_string());

    let listener = TcpListener::bind(&listen_addr).await?;
    println!(
        "WebSocket bridge listening on ws://{} -> staging tcp://{}",
        listen_addr, staging_addr
    );

    loop {
        let (stream, peer) = listener.accept().await?;
        println!("WS client connected: {}", peer);

        let staging = staging_addr.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, staging).await {
                eprintln!("WS client {} error: {}", peer, e);
            } else {
                println!("WS client {} disconnected", peer);
            }
        });
    }
}
