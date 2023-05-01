use std::ops::{Index, IndexMut, Deref};

use tokio::sync::broadcast::{self, Receiver};

use crate::Update;

#[derive(Debug)]
pub struct SubscribableVec<T> {
    vec: Vec<T>,
    updates: broadcast::Sender<Update<usize>>,
}

impl<T> Deref for SubscribableVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl<T> Default for SubscribableVec<T> {
    fn default() -> Self {
        let (updates, _) = broadcast::channel(16);
        Self { vec: Default::default(), updates }
    }
}

impl<T> SubscribableVec<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let (updates, _) = broadcast::channel(16);
        Self {
            updates,
            vec: Vec::with_capacity(capacity)
        }
    }
    
    pub fn with_broadcast_capacity(capacity: usize) -> Self {
        let (updates, _) = broadcast::channel(capacity);
        Self {
            updates,
            vec: Vec::new()
        }
    }

    pub fn with_vec_and_broadcast_capacity(vec_capacity: usize, broadcast_capacity: usize) -> Self {
        let (updates, _) = broadcast::channel(broadcast_capacity);
        Self {
            updates,
            vec: Vec::with_capacity(vec_capacity)
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if let Some(val) = self.vec.get_mut(index) {
            _ = self.updates.send(Update::Mutation(index));
            Some(val)
        } else {
            None
        }
    }

    pub fn push(&mut self, value: T) {
        _ = self.updates.send(Update::Addition(self.vec.len()));
        self.vec.push(value);
    }

    pub fn pop(&mut self) -> Option<T> {
        let res = self.vec.pop();
        _ = self.updates.send(Update::Deletion(self.vec.len()));
        res
    }

    pub fn remove(&mut self, index: usize) -> T {
        let val = self.vec.remove(index);
        _ = self.updates.send(Update::Deletion(index));
        val
    }

    pub fn insert(&mut self, index: usize, element: T) {
        self.vec.insert(index, element);
        let _ = self.updates.send(Update::Addition(index));
    }

    pub fn subscribe(&self) -> Receiver<Update<usize>> {
        self.updates.subscribe()
    }
}

impl<T> Index<usize> for SubscribableVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.vec[index]
    }
}

impl<T> IndexMut<usize> for SubscribableVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let val = &mut self.vec[index];
        _ = self.updates.send(Update::Mutation(index));
        val
    }
}


#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};
    use tokio::sync::{RwLock, broadcast::Receiver};
    use super::*;

    #[tokio::test]
    async fn test_subscription() {
        let sub_vec = Arc::new(RwLock::new(SubscribableVec::<i32>::new()));
        let m1 = sub_vec.clone();
        let m2 = sub_vec.clone();
        let r1 = sub_vec.read().await.subscribe();
        let r2 = sub_vec.read().await.subscribe();
        sub_vec.read().await.len();

        async fn test_subscription_results(mut rec: Receiver<Update<usize>>, map: Arc<RwLock<SubscribableVec<i32>>>) {
            match rec.recv().await {
                Ok(Update::Addition(val)) => {
                    assert_eq!(val, 0);
                    assert_eq!(map.read().await.get(0), Some(&2));
                },
                e => assert!(false, "Wrong result {e:?}")
            };
            match rec.recv().await {
                Ok(Update::Mutation(val)) => {
                    assert_eq!(val, 0);
                    assert_eq!(map.read().await.get(0), Some(&3));
                },
                e => assert!(false, "Wrong result {e:?}")
            };
            match rec.recv().await {
                Ok(Update::Addition(val)) => {
                    assert_eq!(val, 1);
                    assert_eq!(map.read().await.get(1), Some(&4));
                },
                e => assert!(false, "Wrong result {e:?}")
            };
            match rec.recv().await {
                Ok(Update::Deletion(val)) => {
                    assert_eq!(val, 0);
                    assert_eq!(map.read().await.get(0), Some(&4));
                },
                e => assert!(false, "Wrong result {e:?}")
            };
            match rec.recv().await {
                Ok(Update::Deletion(val)) => {
                    assert_eq!(val, 0);
                    assert_eq!(map.read().await.get(0), None);
                },
                e => assert!(false, "Wrong result {e:?}")
            };
        }

        let j1 = tokio::spawn(async move {
            test_subscription_results(r1, m1).await
        });
        let j2 = tokio::spawn(async move {
            test_subscription_results(r2, m2).await
        });
        sub_vec.write().await.push(2);
        // Give the locks time to verify that the value was acutally mutated
        tokio::time::sleep(Duration::from_millis(20)).await;
        *sub_vec.write().await.get_mut(0).unwrap() = 3;
        tokio::time::sleep(Duration::from_millis(20)).await;
        sub_vec.write().await.insert(1, 4);
        tokio::time::sleep(Duration::from_millis(20)).await;
        sub_vec.write().await.remove(0);
        tokio::time::sleep(Duration::from_millis(20)).await;
        sub_vec.write().await.pop();
        tokio::try_join!(j1, j2).unwrap();
    }
}
