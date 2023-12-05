use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use bytes::Bytes;
use mini_redis::{Command, Frame, Result};
use tokio::net::{TcpListener, TcpStream};
type Db = Arc<Mutex<HashMap<String, Bytes>>>;
// Sharding is a way to split a database into multiple parts called shards. Each shard is a separate database. The goal is to distribute the load across multiple servers. Each server is responsible for one or more shards, but not all of them.
type ShardedDb = Arc<Vec<Mutex<HashMap<String, Bytes>>>>;

// Create a new sharded db
fn new_sharded_db(n: usize) -> ShardedDb {
    let mut dbs = Vec::with_capacity(n);
    for _ in 0..n {
        dbs.push(Mutex::new(HashMap::new()));
    }
    Arc::new(dbs)
}
#[tokio::main]
async fn main() -> Result<()> {
    // bind a listener to the address
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    println!("Listening");

    // The `db` object is wrapped in a `Mutex` to allow sharing it between
    // multiple tasks. The `Arc` is required to make it sendable between
    // threads.

    // Difference between std::sync::Mutex and tokio::sync::Mutex is that
    // std::sync::Mutex is blocking the entire thread therefore all the tasks on that thread will be blocked

    // tokio::sync::Mutex is only blocks the task that is trying to access the resource and not the entire thread, when the Mutex is locked, the task will be yield back to the scheduler and the scheduler will schedule other tasks to run.
    let db:Db = Arc::new(Mutex::new(HashMap::new()));

    // let db = new_sharded_db(16);

    // loop forever, accepting connections
    loop {
        // The first item contains the socket and address of the new connection.
        // The second item contains the IP and port of the new connection.
        let (socket, _) = listener.accept().await?;

        // Clone the handle to the hash map.
        let db = db.clone();

        // // Single connection
        // process(socket).await;

        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
    Ok(())
}

async fn process(socket: TcpStream, db:Db)  {
    // The `Connection` lets us read/write redis **frames** instead of
    // byte streams. The `Connection` type is defined by mini-redis.
    let mut connection = mini_redis::Connection::new(socket);
    // Use `read_frame` to receive a command from the connection.
    while let Some(frame) = connection.read_frame().await.unwrap(){
        let response = match Command::from_frame(frame).unwrap(){
            Command::Set(cmd) => {
                let mut db = db.lock().unwrap();
                // The value is stored as `Vec<u8>`
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Command::Get(cmd) => {
                let db = db.lock().unwrap();
                if let Some(value) = db.get(cmd.key()){
                    // `Frame::Bulk` expects data to be of type `Bytes`.
                    Frame::Bulk(value.clone())
                }else{
                    // Return "Null" data
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };
        // Write the response to the client
        connection.write_frame(&response).await.unwrap();
    }
}