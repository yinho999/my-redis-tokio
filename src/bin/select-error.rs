use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    let (tx, rx): (tokio::sync::oneshot::Sender<io::Result<String>>, tokio::sync::oneshot::Receiver<io::Result<String>>) = oneshot::channel();
    let listener = TcpListener::bind("localhost:3465").await?;
    tokio::spawn(async move {
        tx.send(Ok("Hello".to_string())).unwrap();
    });

    tokio::select! {
        res = async {
            loop {
                // This Result will propagated to the res <pattern>
                let (socket, _) = listener.accept().await?;
                tokio::spawn(async move { process(socket).await });
            }

            // This Result will propagated to the res <pattern>
            // Help the rust type inferencer out
            Ok::<_, io::Error>(())
        } => {
            // This Result will propagated to the select! macro expression
            res?;
        }
        _ = rx => {
            println!("terminating accept loop");
        }
    }

    Ok(())
}

async fn process(socket: TcpStream) {
    // The `Connection` lets us read/write redis **frames** instead of
    // byte streams. The `Connection` type is defined by mini-redis.
    let mut connection = mini_redis::Connection::new(socket);
}

