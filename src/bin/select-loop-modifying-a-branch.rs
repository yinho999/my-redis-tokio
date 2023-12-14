async fn action(input: Option<i32>) -> Option<String> {
    // If the input is `None`, return `None`.
    // This could also be written as `let i = input?;`
    let i = match input {
        Some(input) => input,
        None => return None,
    };
    // async logic here
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    Some(format!("Done {}", i))
}

#[tokio::main]
async fn main() {
    let (mut tx, mut rx) = tokio::sync::mpsc::channel(128);

    let mut done = false;
    let operation = action(None);
    tokio::pin!(operation);

    tokio::spawn(async move {
        let _ = tx.send(1).await;
        let _ = tx.send(2).await;
        let _ = tx.send(3).await;
        let _ = tx.send(4).await;
        let _ = tx.send(5).await;
        let _ = tx.send(6).await;
    });

    // There is the logic - remember when one branch is executed, the other branches will be aborted.
    loop {
        tokio::select! {
            // 1. Wait for rx to receive a value which is even
            // 2. Set the async operation with the even value
            Some(v) = rx.recv() => {
                if v % 2 == 0 {
                    // `.set` is a method on `Pin`.
                    operation.set(action(Some(v)));
                    done = false;
                }
            }
            // 3. Wait for the async operation to complete, in the same time listen for more even values on the channel
            // 4. If a new even value is received, abort the existing operation and start over with the new even value. Since the operation is aborted, the async operation will not be polled.
            // if !done is a branch precondition which runs before async expression, it will disable this branch if the condition is false. In this case when done is true, the async expression will not be polled.
            // If operation was polled again after completing, it would panic with a "poll after Complete" error. The if !done condition prevents this from happening by disabling the operation branch of the select! macro once operation has completed 4.
            res = &mut operation, if !done => {
                done = true;
                println!("GOT = {:?}", res);
            }
            else => {
                println!("All channels have been closed.");
                return;
            }
        }
    }
}