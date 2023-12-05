use std::io::Cursor;
use bytes::{BufMut, BytesMut};
use tokio::net::TcpStream;
use tokio::io::AsyncReadExt;
use mini_redis::{Frame, Result};

pub struct Connection {
    stream: TcpStream,
    buffer: Vec<u8>,
    cursor: usize,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream,
            // buffer with 4kb capacity
            buffer: vec![0; 4096],
            cursor: 0,
        }
    }
    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            // Attempt to parse a frame from the buffered data. If enough data has been buffered, the frame is returned.
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }
            // Ensure the buffer has capacity
            if self.cursor == self.buffer.len() {
                // Grow the buffer if needed
                self.buffer.resize(self.cursor * 2, 0);
            }
            // Read into the buffer, tracking the number of bytes read
            let n = self.stream.read(&mut self.buffer[self.cursor..]).await?;

            // If `n` is 0, the remote closed the connection.
            if n == 0 {
                if self.cursor == 0 {
                    // No data read, return `Ok(None)`
                    return Ok(None);
                } else {
                    // The peer closed the socket while sending a frame. This is invalid, so return an error.
                    return Err("connection reset by peer".into());
                }
            } else {
                // Increment the cursor by the number of bytes read
                self.cursor += n;
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
                unsafe { self.buffer.advance_mut(len); }

                // Return the frame to the caller
                Ok(Some(frame))

                // endregion
            }
            // Not enouch data has been buffered
            Err(Incomplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
        // endregion
    }
}

