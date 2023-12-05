use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// https://github.com/tokio-rs/website/blob/master/tutorial-code/io/src/echo-server-copy.rs
#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:6142").await?;
    // Split the socket into reader handler and writer handler
    // Any type which implements both AsyncRead + AsyncWrite can split into reader and writer

    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            // region Copy data via Manual buffer
            let mut buf = vec![0; 1024];
            loop {
                match socket.read(&mut buf).await {
                    // socket closed when `Ok(0)` is received
                    Ok(0) => return,
                    // Copy data back to socket when data is received
                    Ok(n) => {
                        if socket.write_all(&buf[..n]).await.is_err() {
                            // Unexpected socket error. There isn't much we can do here so just stop processing.
                            return;
                        }
                    }
                    Err(_) => {
                        // Unexpected socket error. There isn't much we can do here so just stop processing.
                        return;
                    }
                }
            }
            // endregion
        });
    }
}