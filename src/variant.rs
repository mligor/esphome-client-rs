use crate::result::Result;

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub trait VarintWrite {
    fn write_varint(&mut self, value: u32) -> WriteVarint<'_, Self>
    where
        Self: AsyncWrite + Unpin + Sized;
}

pub trait VarintRead {
    fn read_varint(&mut self) -> ReadVarint<'_, Self>
    where
        Self: AsyncRead + Unpin + Sized;
}

pub struct WriteVarint<'a, W>
where
    W: AsyncWrite + Unpin + ?Sized,
{
    writer: &'a mut W,
    value: u32,
}

pub struct ReadVarint<'a, R>
where
    R: AsyncRead + Unpin + ?Sized,
{
    reader: &'a mut R,
}

impl<W> VarintWrite for W
where
    W: AsyncWrite + Unpin + Sized,
{
    fn write_varint(&mut self, value: u32) -> WriteVarint<'_, Self> {
        WriteVarint { writer: self, value }
    }
}

impl<R> VarintRead for R
where
    R: AsyncRead + Unpin + Sized,
{
    fn read_varint(&mut self) -> ReadVarint<'_, Self> {
        ReadVarint { reader: self }
    }
}

impl<'a, W: AsyncWrite + Unpin + ?Sized> Future for WriteVarint<'a, W> {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        while self.value >= 0x80 {
            let byte_to_write = ((self.value as u8) & 0x7F) | 0x80;
            match Pin::new(&mut self.writer).poll_write(cx, &[byte_to_write]) {
                Poll::Ready(Ok(_)) => self.value >>= 7,
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e.into())),
                Poll::Pending => return Poll::Pending,
            }
        }

        // Now write the final byte
        let byte_to_write = self.value as u8;
        match Pin::new(&mut self.writer).poll_write(cx, &[byte_to_write]) {
            Poll::Ready(Ok(_)) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<'a, R: AsyncRead + Unpin + ?Sized> Future for ReadVarint<'a, R> {
    type Output = Result<u32>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut varint = 0u32;
        let mut shift = 0;
        let mut byte_buf = [0u8; 1];
        let mut read_buf = ReadBuf::new(&mut byte_buf);

        loop {
            let _ = Pin::new(&mut self.reader).poll_read(cx, &mut read_buf)?;

            if read_buf.filled().is_empty() {
                // No more bytes to read
                return Poll::Ready(Err("End of stream".into()));
            }

            let byte = read_buf.filled()[0];
            varint |= ((byte & 0x7F) as u32) << shift;

            if byte & 0x80 == 0 {
                // This was the last byte of the varint
                return Poll::Ready(Ok(varint));
            }

            shift += 7;
            if shift > 32 {
                return Poll::Ready(Err("Varint too long".into()));
            }

            // Clear the buffer for the next byte
            read_buf.clear();
        }
    }
}
