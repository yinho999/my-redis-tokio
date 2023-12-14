use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx1, mut rx1) = mpsc::channel(128);
    let (tx2, mut rx2) = mpsc::channel(128);
    let (tx3, mut rx3) = mpsc::channel(128);
    tokio::spawn(async move {
        // Send values on `tx1` and `tx2`.
        tx1.send(Some("one")).await.unwrap();
        tx2.send(Some("two")).await.unwrap();
        tx3.send(Some("three")).await.unwrap();
        // Sleep for 1 second
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        // Drop tx1, tx2 and tx3 in order to close the channels
        tx1.send(Some("one v2")).await.unwrap();
        drop(tx1);
        tx2.send(Some("two v2")).await.unwrap();
        drop(tx2);
        tx3.send(Some("three v2")).await.unwrap();
        drop(tx3);
    });

    // When a channel is closed, None is returned, then select! expression will continue to wait for the value from the other channel.
    // If all channels are closed, the else branch will be executed and the loop will be terminated.
    // The `select!` macro randomly selects one of the branches to check first for readiness. When multiple branches are ready, one of them is chosen at random.
    // This is to prevent reading from one channel and ignoring the other channels.
    // When multiple branches have values pending, only one channel has a value popped off. All other channels will have their values remain in the channel until the next iteration of the loop. No message is lost.
    loop {
        let msg = tokio::select! {
            Some(msg) = rx1.recv() => msg,
            Some(msg) = rx2.recv() => msg,
            Some(msg) = rx3.recv() => msg,
            else => { break }
        };

        println!("Got {:?}", msg);
    }

    println!("All channels have been closed.");
}