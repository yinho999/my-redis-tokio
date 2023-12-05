use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::error::Error;
use tokio::io;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let socket = TcpStream::connect("127.0.0.1:6142").await?;
    let (mut rd, mut wr) = io::split(socket);
    // Write some data in the background
    tokio::spawn(async move {
        wr.write_all(b"hello\r\n").await?;
        wr.write_all(b"world\r\n").await?;
        // Need to help the compiler understand the type of the error
        Ok::<_, io::Error>(())
    });

    let mut buf = vec![0; 128];
    loop {
        let n = rd.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        println!("GOT {:?}", &buf[..n]);
        // to char
        let buf = std::str::from_utf8(&buf[..n]).unwrap();
        println!("GOT {:?}", buf);
    }

    Ok(())
}