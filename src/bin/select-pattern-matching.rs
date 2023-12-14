use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (mut tx1, mut rx1): (tokio::sync::mpsc::Sender<Option<String>>, tokio::sync::mpsc::Receiver<Option<String>>) = mpsc::channel(128);
    let (mut tx2, mut rx2): (tokio::sync::mpsc::Sender<Option<String>>, tokio::sync::mpsc::Receiver<Option<String>>) = mpsc::channel(128);

    tokio::spawn(async move {
        // Drop tx1 and tx2 in order to close the channels
        drop(tx1);
        drop(tx2);
    });

    // `tokio::select!` expression waits for the value from either rx1.recv() or rx2.recv()
    // Some(T) is returned if the channel is not closed
    // If the channel is closed, None is returned and it does not match the pattern, therefore the select! expression will continue to wait for the value from the other channel
    tokio::select! {
        Some(v) = rx1.recv() => {
            println!("Got {:?} from rx1", v);
        }
        Some(v) = rx2.recv() => {
            println!("Got {:?} from rx2", v);
        }
        else => {
            println!("Both channels closed");
        }
    }
}