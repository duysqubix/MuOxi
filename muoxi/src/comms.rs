//!
//! File: comms.rs
//! Usage: communication, socket control
//!

// #[deny(missing_docs)]
#![allow(dead_code)]

use futures::{SinkExt, Stream, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio_util::codec::{Framed, LinesCodec, LinesCodecError};

type Tx = mpsc::UnboundedSender<String>;
type Rx = mpsc::UnboundedReceiver<String>;

#[derive(Debug)]
pub enum Message {
    /// A message that was recieved from a connected client
    Recieved(String),

    /// A message that needs to be sent to clients
    Broadcast(String),
}

pub struct ClientShared {
    /// The server handle for all connected clients
    /// It contains each unique 'Tx' portion of the
    /// mpsc channel for communication between server
    /// and clients.
    pub clients: HashMap<SocketAddr, Tx>,
}

impl ClientShared {
    /// create a new, empty instance of this interal client *book*
    pub fn new() -> Self {
        ClientShared {
            clients: HashMap::new(),
        }
    }

    pub async fn broadcast(&mut self, sender: SocketAddr, message: &str) {
        for client in self.clients.iter_mut() {
            if *client.0 != sender {
                let _ = client.1.send(message.into());
            }
        }
    }
}

pub struct Client {
    /// The TCP Socket that represents the connected
    /// instance from the network, wrapped in a codec
    /// so we can work with lines instead of raw byte
    /// operations
    pub lines: Framed<TcpStream, LinesCodec>,
    /// Recieve half of the message channel
    ///
    /// Used to recieve message from other connected clients. When
    /// a message is recieved off this `Rx` it will be redirected
    /// to the connected clients socket.
    rx: Rx,
}

impl Client {
    /// Create a new instance of a connected client
    pub async fn new(
        state: Arc<Mutex<ClientShared>>,
        lines: Framed<TcpStream, LinesCodec>,
    ) -> std::io::Result<Client> {
        let addr = lines.get_ref().peer_addr()?;

        let (tx, rx) = mpsc::unbounded_channel();

        state.lock().await.clients.insert(addr, tx);

        Ok(Client {
            lines: lines,
            rx: rx,
        })
    }
}

/// Stream implementation of Client struct.
/// This polls both the `Rx`, and the `Framed` types
/// A message is constructed whenever an even is ready until the
/// `Framed` stream returns `None`
impl Stream for Client {
    type Item = Result<Message, LinesCodecError>;

    /// First poll the `UnboundedReciever`, the internal Rx for each
    /// connected client. Information here will eventually be redirectd to the clients sockets
    ///
    /// Secondly poll the `Framed` stream to see if any information has been sent
    /// to the server by the client, this information will be parsed and server will
    /// act accordingly.
    ///
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Poll RX
        if let Poll::Ready(Some(v)) = self.rx.poll_next_unpin(cx) {
            return Poll::Ready(Some(Ok(Message::Recieved(v))));
        }

        // Poll Stream
        let result: Option<_> = futures::ready!(self.lines.poll_next_unpin(cx));
        Poll::Ready(match result {
            Some(Ok(msg)) => Some(Ok(Message::Broadcast(msg))),
            Some(Err(e)) => Some(Err(e)),
            None => None,
        })
    }
}
