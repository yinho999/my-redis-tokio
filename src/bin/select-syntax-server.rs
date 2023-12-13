use std::io;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

#[tokio::main]
async fn main() -> io::Result<()> {
    let (tx, rx) = oneshot::channel();

    tokio::spawn(async move {
        tx.send(()).unwrap();
    });

    let mut listener = TcpListener::bind("localhost:3465").await?;

    // <pattern> = <async expression> => <handler>
    // pattern is the value returned by the async expression
    // handler is the code that will be executed when the async expression completes
    // The async block will complete when either the listener accepts a connection or the rx channel receives a message.
    tokio::select! {
        // The loop runs until there is an error, or the rx channel receives a message.
        // _ means that we don't care about the value returned by the async expression
        _ = async {
            loop {
                let (socket, _) = listener.accept().await?;
                tokio::spawn(async move { process(socket).await });
            }

            // Help the rust type inferencer out
            Ok::<_, io::Error>(())
        } => {}
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

