use tokio::sync::oneshot;
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use std::io;

// Immutable state - the data variable is immutablly borrowed by both async expressions.
// If one of the async expression succeeds, the other one will be dropped.
// If one of them fails, the other one will be continued.
async fn race(
    data: &[u8],
    addr1: SocketAddr,
    addr2: SocketAddr
) -> io::Result<()> {
    tokio::select! {
        Ok(_) = async {
            let mut socket = TcpStream::connect(addr1).await?;
            socket.write_all(data).await?;
            Ok::<_, io::Error>(())
        } => {}
        Ok(_) = async {
            let mut socket = TcpStream::connect(addr2).await?;
            socket.write_all(data).await?;
            Ok::<_, io::Error>(())
        } => {}
        else => {}
    };

    Ok(())
}


// Mutable state - `tokio::select!` guarantees that only one of the async expressions will be executed at a time. Therefore each <handler> may mutably borrow the data variable.
// If one of the async expression succeeds, the other one will be dropped.
// If one of them fails, the other one will be continued.
#[tokio::main]
async fn main() {
    let (tx1, rx1) = oneshot::channel();
    let (tx2, rx2) = oneshot::channel();

    let mut out = String::new();

    tokio::spawn(async move {
        // Send values on `tx1` and `tx2`.
        tx1.send("one").unwrap();
        tx2.send("two").unwrap();
    });

    tokio::select! {
        _ = rx1 => {
            out.push_str("rx1 completed");
        }
        _ = rx2 => {
            out.push_str("rx2 completed");
        }
    }

    println!("{}", out);
}