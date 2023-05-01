# Subscribables

[![crates.io](https://img.shields.io/badge/crates.io-v0.1.0-orange)](https://crates.io/crates/subscribables)

This crate provides wrappers around [`Vec`](https://doc.rust-lang.org/alloc/vec/struct.Vec.html)
and [`HashMap`](https://doc.rust-lang.org/std/collections/hash_map/struct.HashMap.html)
that supports subscribing to mutations via a tokio [`broadcast::channel`](https://docs.rs/tokio/latest/tokio/sync/index.html#broadcast-channel).

## Example
```rust, no_run
use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;
use subscribables::{SubscribableVec, Update};

#[tokio::main]
async fn main() {
    let shared_vec = Arc::new(RwLock::new(SubscribableVec::new(20)));
    let vec_ref = shared_vec.clone();
    let handle = tokio::spawn(async move {
        wait_for_n_results(10, vec_ref).await
    });
    // Give the thread time to aquire the lock once 
    tokio::time::sleep(Duration::from_millis(20)).await;
    let mut vec = shared_vec.write().await;
    for i in 0..20 {
        vec.push(i);
    }
    handle.await.unwrap();
}

async fn wait_for_n_results(n: usize, data: Arc<RwLock<SubscribableVec<i32>>>) {
    let (mut current_len, mut updates) = {
        let read = data.read().await;
        (read.len(), read.subscribe())
    };
    if current_len >= n {
        println!("We have {n} or more results already returning.");
        return;
    }
    while let Ok(update) = updates.recv().await {
        match update {
            Update::Addition(..) => current_len += 1,
            Update::Deletion(..) => current_len -= 1,
            Update::Mutation(..) => {
                // We dont care about mutations in this case
            },
        };
        if current_len >= n {
            println!("We have {n} or more updates now returning.");
            return;
        }
    }
}
```
# Note:
Setting the right broadcast capacity for your needs is **very** important as overflowing the
buffer can lead to very hard to debug errors.
In general you should set the channel buffer size to the maximum number of mutations you
will do inside a single syncronous block of code.
This way tokio will be able to schedule time for the reciever to recieve the updates from the buffer.
To illustrate this point even a buffer of size 1 is sufficient if you do a `tokio::time::sleep` after each mutation.
