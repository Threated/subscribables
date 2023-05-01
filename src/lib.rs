#![doc = include_str!("../Readme.md")]
#![deny(missing_docs)]

mod map;
mod vec;
mod update;

pub use update::Update;
pub use map::SubscribableMap;
pub use vec::SubscribableVec;

#[cfg(test)]
mod example {
    use std::{sync::Arc, time::Duration};

    use tokio::sync::RwLock;

    use super::{SubscribableVec, Update};

    #[tokio::test]
    async fn main() {
        let shared_vec = Arc::new(RwLock::new(SubscribableVec::new(20)));
        let vec_ref = shared_vec.clone();
        let handle = tokio::spawn(async move {
            wait_for_n_results(10, vec_ref).await
        });
        // Give the thread time to aquire the lock once 
        tokio::time::sleep(Duration::from_millis(60)).await;
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
                return;
            }
        }
        panic!("Did not reach n elements");
    }
}
