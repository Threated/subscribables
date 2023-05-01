
use std::{collections::HashMap, hash::Hash, ops::{Index, Deref}};

use tokio::sync::broadcast;

use crate::Update;

#[derive(Debug, Clone)]
pub struct SubscribableMap<K, V>
where
    K: Clone,
{
    map: HashMap<K, V>,
    updates: broadcast::Sender<Update<K>>,
}

impl<K, V> Deref for SubscribableMap<K, V>
where
    K: Clone,
{
    type Target = HashMap<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}


impl<K: Clone, V> Default for SubscribableMap<K, V> {
    fn default() -> Self {
        let (updates, _) = broadcast::channel(16);
        Self {
            map: Default::default(),
            updates,
        }
    }
}

impl<K, V> SubscribableMap<K, V>
where
    K: Hash + Eq + Clone,
{
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
            ..Default::default()
        }
    }

    pub fn with_broadcast_capacity(capacity: usize) -> Self {
        let (updates, _) = broadcast::channel(capacity);
        Self {
            map: Default::default(),
            updates,
        }
    }

    pub fn with_map_and_broadcast_capacity(map_capacity: usize, broadcast_capacity: usize) -> Self {
        let (updates, _) = broadcast::channel(broadcast_capacity);
        Self {
            map: HashMap::with_capacity(map_capacity),
            updates,
        }
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        if let Some(v) = self.map.get_mut(key) {
            _ = self.updates.send(Update::Mutation(key.clone()));
            Some(v)
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.map.contains_key(&key) {
            Some(value)
        } else {
            self.map.insert(key.clone(), value);
            _ = self.updates.send(Update::Addition(key));
            None
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(v) = self.map.remove(key) {
            _ = self.updates.send(Update::Deletion(key.clone()));
            Some(v)
        } else {
            None
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Update<K>> {
        self.updates.subscribe()
    }
}

impl<K: Clone + Hash + Eq, V> Index<&K> for SubscribableMap<K, V> {
    type Output = V;

    fn index(&self, index: &K) -> &Self::Output {
        &self.map[index]
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};
    use tokio::sync::{RwLock, broadcast::Receiver};
    use super::*;

    #[tokio::test]
    async fn test_subscription() {
        let map = Arc::new(RwLock::new(SubscribableMap::<i32, i32>::new()));
        let m1 = map.clone();
        let m2 = map.clone();
        let r1 = map.read().await.subscribe();
        let r2 = map.read().await.subscribe();

        async fn test_subscription_results(mut rec: Receiver<Update<i32>>, map: Arc<RwLock<SubscribableMap<i32, i32>>>) {
            match rec.recv().await {
                Ok(Update::Addition(val)) => {
                    assert_eq!(val, 1);
                    assert_eq!(map.read().await.get(&1), Some(&2));
                },
                e => assert!(false, "Wrong result {e:?}")
            };
            match rec.recv().await {
                Ok(Update::Mutation(val)) => {
                    assert_eq!(val, 1);
                    assert_eq!(map.read().await.get(&1), Some(&3));
                },
                e => assert!(false, "Wrong result {e:?}")
            };
            match rec.recv().await {
                Ok(Update::Deletion(val)) => {
                    assert_eq!(val, 1);
                    assert_eq!(map.read().await.get(&1), None);
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
        map.write().await.insert(1, 2);
        // Give the locks time to verify that the value was acutally mutated
        tokio::time::sleep(Duration::from_millis(20)).await;
        *map.write().await.get_mut(&1).unwrap() = 3;
        tokio::time::sleep(Duration::from_millis(20)).await;
        map.write().await.remove(&1);
        tokio::try_join!(j1, j2).unwrap();
    }
}


