use futures::ready;
use log::{info, warn};
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::str;
use std::task::{Context, Poll};
use tokio::io;

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
                    .map(|string| info!("{}->{}: {} ", me.from, me.to, string))
                    .map_err(|e| warn!("{}", e));
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
