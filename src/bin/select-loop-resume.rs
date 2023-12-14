async fn action() {
    // Some asynchronous logic
}

#[tokio::main]
async fn main() {
    let (mut tx, mut rx) = tokio::sync::mpsc::channel(128);

    // Has been called without await
    let operation = action();

    // Pin the operation to the stack
    /*
        Without pinning, the compiler will complain:
        error[E0599]: no method named `poll` found for struct
         `std::pin::Pin<&mut &mut impl std::future::Future>`
         in the current scope
          --> src/main.rs:16:9
           |
        16 | /         tokio::select! {
        17 | |             _ = &mut operation => break,
        18 | |             Some(v) = rx.recv() => {
        19 | |                 if v % 2 == 0 {
        ...  |
        22 | |             }
        23 | |         }
           | |_________^ method not found in
           |             `std::pin::Pin<&mut &mut impl std::future::Future>`
           |
           = note: the method `poll` exists but the following trait bounds
            were not satisfied:
           `impl std::future::Future: std::marker::Unpin`
           which is required by
           `&mut impl std::future::Future: std::future::Future`
     */
    tokio::pin!(operation);


    tokio::spawn(async move {
        loop {
            tx.send(1).await.unwrap();
        }
    });
    loop {
        tokio::select! {
            // awaiting a reference, the value being referenced should be pinned or implement `Unpin`
            _ = &mut operation => break,
            Some(v) = rx.recv() => {
                if v % 2 == 0 {
                    break;
                }
            }
        }
    }
}