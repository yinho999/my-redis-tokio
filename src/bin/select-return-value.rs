async fn computation1() -> String {
    // .. computation
    "Computation 1".to_string()
}

async fn computation2() -> String {
    // .. computation
    "Computation 2".to_string()
}

#[tokio::main]
async fn main() {
    // The `tokio::select!` macro returns the value of the first complete <handler> expression.
    // All <handler> expressions must return the same type.
    let out = tokio::select! {
        res1 = computation1() => res1,
        res2 = computation2() => res2,
    };

    println!("Got = {}", out);
}