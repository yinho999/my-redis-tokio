use bytes::Bytes;
use tokio::sync::oneshot;

/// Multiple different commands are multiplexed over a single channel.
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        value: Bytes,
        resp: Responder<()>,
    },
}

// Provide by the requester and used by the manager task to send the command back to the requester
/// The `oneshot::Sender` is used to send the command response back to the requester.
type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[tokio::main]
async fn main() {

    // Create a new channel with a capacity of at most 32.
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    let tx2 = tx.clone();

    // Establish a connection to the server
    // let mut client = mini_redis::client::connect("127.0.0.1:6379").await.unwrap();

    // Spawn a task to set a value
    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Set {
            key: "foo".to_string(),
            value: "bar".into(),
            resp: resp_tx,
        };
        // The `Sender` handles are moved into the task.
        tx.send(cmd).await.expect("TODO: panic message");

        // Await the response
        let res = resp_rx.await;
        println!("GOT = {:?}", res);
    });

    // Spawn a task that sets a value
    let t2 = tokio::spawn(async move {
        // Create a oneshot channel for the response
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Get {
            key: "foo".to_string(),
            resp: resp_tx,
        };
        tx2.send(cmd).await.expect("TODO: panic message");

        // Await the response
        let res = resp_rx.await;
        println!("GOT = {:?}", res);
    });

    // Spawn a manager task that receives commands and send to the server
    let manager = tokio::spawn(async move {
        let mut client = mini_redis::client::connect("127.0.0.1:6379").await.unwrap();
        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Get { key, resp } => {
                    let res = client.get(&key).await;
                    let _ = resp.send(res);
                }
                Command::Set { key, value, resp } => {
                    let res = client.set(&key, value).await;
                    let _ = resp.send(res);
                }
            }
        }
    });
    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap()
}