use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// https://github.com/tokio-rs/website/blob/master/tutorial-code/io/src/echo-server-copy.rs
#[tokio::main]
async fn main() -> io::Result<()>{
    let listener = tokio::net::TcpListener::bind("127.0.0.1:6142").await?;
    // Split the socket into reader handler and writer handler
    // Any type which implements both AsyncRead + AsyncWrite can split into reader and writer

    loop{
        let (mut socket,_) = listener.accept().await?;
        tokio::spawn(async move{
            // region Copy data via splitting the socket into reader and writer

            // reader and writer needs to but the same task
            let (mut rd,mut wr) = socket.split();
            if io::copy(&mut rd,&mut wr).await.is_err(){
                eprintln!("failed to copy");
            }
            // endregion
        });
    }
}