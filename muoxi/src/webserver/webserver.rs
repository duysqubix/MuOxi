//! WebSocket-to-TCP bridge. Listens on `127.0.0.1:8080` for browser clients
//! and, for each WebSocket connection, opens a fresh outbound TCP connection
//! to the staging proxy at `127.0.0.1:8000`. Text messages are forwarded both
//! ways (line-oriented).
//!
//! Plain HTTP GET (no `Upgrade: websocket` header) on the same port serves a
//! minimal in-browser test client (`resources/web/index.html`) so a developer
//! can `docker compose up` and point a browser at the port without needing a
//! separate WS tool.
//!
//! Run after starting `muoxi_server`.
//!
//! ```bash
//! cargo run --bin muoxi_server   # in one terminal
//! cargo run --bin muoxi_web      # in another
//! # then either point a browser at http://127.0.0.1:8080
//! # or connect any WS client to ws://127.0.0.1:8080
//! ```

use futures_util::{SinkExt, StreamExt};
use std::env;
use std::error::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

type AnyError = Box<dyn Error + Send + Sync>;

const INDEX_HTML: &str = include_str!("../../../resources/web/index.html");

fn looks_like_ws_upgrade(preview: &str) -> bool {
    preview.lines().any(|line| {
        let lc = line.to_ascii_lowercase();
        lc.starts_with("upgrade:") && lc.contains("websocket")
    })
}

async fn serve_index(mut stream: TcpStream) -> Result<(), AnyError> {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        INDEX_HTML.len(),
        INDEX_HTML,
    );
    stream.write_all(response.as_bytes()).await?;
    let _ = stream.shutdown().await;
    Ok(())
}

async fn handle_client(stream: TcpStream, staging_addr: String) -> Result<(), AnyError> {
    let mut buf = [0u8; 1024];
    let n = stream.peek(&mut buf).await?;
    let preview = std::str::from_utf8(&buf[..n]).unwrap_or("");

    if !looks_like_ws_upgrade(preview) {
        return serve_index(stream).await;
    }

    let ws = accept_async(stream).await?;
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
