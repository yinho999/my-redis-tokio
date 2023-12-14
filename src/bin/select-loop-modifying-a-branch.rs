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
    // We need to initialize the operation to something. Therefore we set it to None.
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
            // without if !done, this branch will failed with "`async fn` resume after completion" error in the first iteration because action(None) is completed in the line 19.
            // the first iteration - immediately return None because the async operation is completed on top. And this will not cause "`async fn` resume after completion" error since the async operation is not polled again before a new operation is set in the first branch.
            // `&mut operation` can only be poll before complete, therefore if the async operation is completed, it shouldnt be polled again. Therefore we disable this branch by if the operation has been polled once and completed. Until a new operator has been set in the first branch, then we can poll the `&mut operation` again.
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