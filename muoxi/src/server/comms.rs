//! Connection-state types: `Server` (shared), `Client` (per-connection),
//! `Comms` (the per-client SocketAddr+Tx pair stored in `Server`), `Message`.

use crate::auth::AuthBuffer;
use crate::prelude::{Rx, Tx};
use crate::states::ConnStates;
use db::utils::UID;
use futures::stream::Stream;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, mpsc};
use tokio_util::codec::{Framed, LinesCodec, LinesCodecError};

/// The types of message recieved
#[derive(Debug)]
pub enum Message {
    /// Message recieved from connected client
    FromClient(String),
    /// Message recieved from shared Rx
    OnRx(String),
}

/// Wrapper around a connected socket. Non-persistent, only valid inside the
/// `process()` lifetime.
#[derive(Debug)]
pub struct Client {
    /// session UID (socket-tied, set in Client::new)
    pub uid: UID,
    /// current state of connected client
    pub state: ConnStates,
    /// encodes and decodes incoming streams
    pub lines: Framed<TcpStream, LinesCodec>,
    /// socket address of connected client
    pub addr: SocketAddr,
    /// authenticated account UID, set on successful login
    pub account_uid: Option<UID>,
    /// selected character UID, set on character-select / character-create
    pub character_uid: Option<UID>,
    /// scratch space for auth state transitions
    pub auth_buffer: AuthBuffer,
    rx: Rx,
}

impl Client {
    /// Asynchronously construct a `Client`. Registers the client into
    /// `Server.clients` under `uid`.
    pub async fn new(
        uid: UID,
        server: Arc<Mutex<Server>>,
        stream: TcpStream,
    ) -> tokio::io::Result<Self> {
        let addr = stream.peer_addr()?;
        let (tx, rx) = mpsc::unbounded_channel();
        let comms = Comms {
            addr,
            tx,
            character_uid: None,
        };
        server.lock().await.clients.insert(uid, comms);
        Ok(Self {
            uid,
            state: ConnStates::AwaitingName,
            lines: Framed::new(stream, LinesCodec::new()),
            addr,
            account_uid: None,
            character_uid: None,
            auth_buffer: AuthBuffer::default(),
            rx,
        })
    }
}

impl Stream for Client {
    type Item = Result<Message, LinesCodecError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if let Poll::Ready(Some(v)) = this.rx.poll_recv(cx) {
            return Poll::Ready(Some(Ok(Message::OnRx(v))));
        }

        match Pin::new(&mut this.lines).poll_next(cx) {
            Poll::Ready(Some(Ok(message))) => Poll::Ready(Some(Ok(Message::FromClient(message)))),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Server-owned structure holding each client's `SocketAddr`, outbound `Tx`,
/// and the UID of the character they're currently playing (if any). The
/// `character_uid` is set when the session transitions into `Playing` and
/// cleared on disconnect — it lets room-scoped broadcast (`say`, future
/// `emote`, etc.) find listeners without holding the per-task `Client` lock.
#[derive(Debug)]
pub struct Comms {
    /// remote socket address of the connected client
    pub addr: SocketAddr,
    /// outbound channel — messages pushed here are delivered to the client's
    /// TCP socket by its `process()` task draining `Client.rx`
    pub tx: Tx,
    /// UID of the character this session is playing, set when entering
    /// `Playing` state; `None` while still in auth / menu states
    pub character_uid: Option<UID>,
}

/// Shared ownership structure between all connected clients.
#[derive(Debug, Default)]
pub struct Server {
    /// Holds information regarding connected clients
    pub clients: HashMap<UID, Comms>,
}

impl Server {
    /// creates shared struct between clients and actual server
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    /// Broadcast `message` to every connected client. The sender (`sender`)
    /// receives a "You broadcasted, ..." echo while everyone else gets the raw
    /// message.
    pub async fn broadcast(&mut self, sender: SocketAddr, message: &str) {
        for (_uid, comms) in self.clients.iter_mut() {
            if comms.addr != sender {
                let _ = comms.tx.send(message.into());
            } else {
                let msg = format!("You broadcasted, {}", message);
                let _ = comms.tx.send(msg);
            }
        }
    }
}
