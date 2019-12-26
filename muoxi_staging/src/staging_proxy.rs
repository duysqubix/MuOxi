//!
//! Main Proxy Staging TCP Client communicates with the Game Engine and relays
//! game engine response to connected clients.
//!

use tokio::io;
use tokio::net::{TcpListener, TcpStream};

use futures::future::try_join;
use futures::{ready, FutureExt};

use std::error::Error;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::str;
use std::task::{Context, Poll};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let listen_addr = "127.0.0.1:8000".to_string();
    let game_addr = "127.0.0.1:4567".to_string();

    println!(
        "TCP Client listening on {} proxying to {}",
        listen_addr, game_addr
    );

    let mut listener = TcpListener::bind(&listen_addr).await?;

    while let Ok((inbound, _)) = listener.accept().await {
        // do proxy work here.

        let proxy = transfer(inbound, game_addr.clone()).map(|r| {
            if let Err(e) = r {
                println!("Failed to transfer; error={}", e);
            }
        });

        tokio::spawn(proxy);
    }

    Ok(())
}

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

    let client_to_server = copy(&mut ri, &mut wo, &inbound_addr, &outbound_addr);
    let server_to_client = copy(&mut ro, &mut wi, &outbound_addr, &inbound_addr);

    try_join(client_to_server, server_to_client).await?;
    Ok(())
}

#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct CopyOver<'a, R: ?Sized, W: ?Sized> {
    reader: &'a mut R,
    read_done: bool,
    writer: &'a mut W,
    pos: usize,
    cap: usize,
    amt: u64,
    buf: Box<[u8]>,
    to: &'a SocketAddr,
    from: &'a SocketAddr,
}

pub fn copy<'a, R, W>(
    reader: &'a mut R,
    writer: &'a mut W,
    from: &'a SocketAddr,
    to: &'a SocketAddr,
) -> CopyOver<'a, R, W>
where
    R: io::AsyncRead + Unpin + ?Sized,
    W: io::AsyncWrite + Unpin + ?Sized,
{
    CopyOver {
        reader,
        read_done: false,
        writer,
        amt: 0,
        pos: 0,
        cap: 0,
        buf: Box::new([0; 2048]),
        to: to,
        from: from,
    }
}

impl<R, W> Future for CopyOver<'_, R, W>
where
    R: io::AsyncRead + Unpin + ?Sized,
    W: io::AsyncWrite + Unpin + ?Sized,
{
    type Output = io::Result<u64>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<u64>> {
        loop {
            // If our buffer is empty, then we need to read some data to
            // continue.
            if self.pos == self.cap && !self.read_done {
                let me = &mut *self;
                let n = ready!(Pin::new(&mut *me.reader).poll_read(cx, &mut me.buf))?;
                if n == 0 {
                    self.read_done = true;
                } else {
                    self.pos = 0;
                    self.cap = n;
                }
            }

            {
                let me = &mut *self;
                let _ = str::from_utf8(&me.buf[me.pos..me.cap - 2])
                    .map(|string| println!("{}->{}: {} ", me.from, me.to, string))
                    .map_err(|e| println!("{}", e));
            }
            // If our buffer has some data, let's write it out!
            while self.pos < self.cap {
                let me = &mut *self;
                let i = ready!(Pin::new(&mut *me.writer).poll_write(cx, &me.buf[me.pos..me.cap]))?;
                if i == 0 {
                    return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "write zero byte into writer",
                    )));
                } else {
                    self.pos += i;
                    self.amt += i as u64;
                }
            }

            // If we've written al the data and we've seen EOF, flush out the
            // data and finish the transfer.
            // done with the entire transfer.
            if self.pos == self.cap && self.read_done {
                let me = &mut *self;
                ready!(Pin::new(&mut *me.writer).poll_flush(cx))?;
                return Poll::Ready(Ok(self.amt));
            }
        }
    }
}
