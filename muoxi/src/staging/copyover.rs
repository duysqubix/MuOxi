//!
//! A custom reimplementation of [tokio::io::copy](https://docs.rs/tokio/0.2.6/tokio/io/fn.copy.html)
//!

use futures::ready;
use log::{info, warn};
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::str;
use std::task::{Context, Poll};
use tokio::io;

/// Asynchronously copies the entire contents of a reader into a writer.
///
/// *difference it echos information being passed from reader to writer*
///
/// This function returns a future that will continuously read data from
/// `reader` and then write it into `writer` in a streaming fashion until
/// `reader` returns EOF.
///
/// On success, the total number of bytes that were copied from `reader` to
/// `writer` is returned.
///
/// This is an asynchronous version of [`std::io::copy`][std].
///
/// # Errors
///
/// The returned future will finish with an error will return an error
/// immediately if any call to `poll_read` or `poll_write` returns an error.
///
/// # Examples
///
/// ```
///
/// # async fn dox() -> std::io::Result<()> {
/// let mut reader: &[u8] = b"hello";
/// let mut writer: Vec<u8> = vec![];
///
/// copy(&mut reader, &mut writer).await?;
///
/// assert_eq!(&b"hello"[..], &writer[..]);
/// # Ok(())
/// # }
/// ```
///
/// [std]: https://doc.rust-lang.org/std/io/fn.copy.html
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
