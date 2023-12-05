use std::io::Cursor;
use bytes::{Buf, BytesMut};
use mini_redis::Frame;
use mini_redis::frame::Error;
use tokio::net::TcpStream;
use mini_redis::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct Connection {
    stream: TcpStream,
    // Using `BytesMut` instead of `Vec<u8>` allows us to avoid copying data when reading from the socket. `BytesMut` is a buffer type from the `bytes` crate. It is similar to `Vec<u8>`, but it has some additional features that make it more suitable for use as a buffer.
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream,
            buffer: BytesMut::with_capacity(4096),
        }
    }
    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            // Attempt to parse a frame from the buffered data. If enough data has been buffered, the frame is returned.
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }
            // There is not enough buffered data to read a frame. Attempt to read more data from the socket.
            // On success, the number of bytes is returned. `0` indicates "end of stream".
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                // The remote closed the connection. For this to be a clean shutdown, there should be no data in the read buffer. If there is, this means that the peer closed the socket while sending a frame. This is invalid, so return an error.
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            }
        }
    }
    fn parse_frame(&mut self) -> Result<Option<Frame>> {
        // region 1. Ensure a full frame is buffered and find the end index of the frame
        // Check the frame using `Frame::check()`
        // Create the `T: Buf` type
        let mut buf = Cursor::new(&self.buffer[..]);
        match Frame::check(&mut buf) {
            Ok(_) => {
                // region 2. Parse the frame
                // Parse the frame using `Frame::parse()`
                // Get the byte length of the frame
                let len = buf.position() as usize;

                // Reset the cursor
                buf.set_position(0);

                // Parse the frame
                let frame = Frame::parse(&mut buf)?;

                // Discard the frame from the buffer
                self.buffer.advance(len);

                // Return the frame to the caller
                Ok(Some(frame))

                // endregion
            }
            // Not enough data has been buffered
            Err(Error::Incomplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
        // endregion
    }
    async fn write_frame(&mut self, frame: &Frame) -> Result<()> {
        // https://redis.io/docs/reference/protocol-spec/
        match frame {
            // Simple Strings
            Frame::Simple(val) => {
                self.stream.write_u8(b'+').await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            // Simple Errors
            Frame::Error(val) => {
                self.stream.write_u8(b'-').await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            // Integers
            Frame::Integer(val) => {
                self.stream.write_u8(b':').await?;
                self.write_decimal(*val).await?;
            }
            // Bulk Strings
            Frame::Bulk(val) => {
                let len = val.len();
                self.stream.write_u8(b'$').await?;
                self.write_decimal(len as u64).await?;
                self.stream.write_all(val).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            // Null Bulk Strings
            Frame::Null => {
                self.stream.write_all(b"$-1\r\n").await?;
            }
            Frame::Array(_val) => unimplemented!("Array"),
        }
        self.stream.flush().await?;
        Ok(())
    }

    /// Write a decimal frame to the stream
    async fn write_decimal(&mut self, val: u64) -> Result<()> {
        use std::io::Write;

        // Convert the value to a string
        let mut buf = [0u8; 12];
        let mut buf = Cursor::new(&mut buf[..]);
        // Write the string to the buffer
        write!(&mut buf, "{}", val)?;
        // Write the string to the stream
        let pos = buf.position() as usize;
        self.stream.write_all(&buf.get_ref()[..pos]).await?;
        self.stream.write_all(b"\r\n").await?;

        Ok(())
    }
}

