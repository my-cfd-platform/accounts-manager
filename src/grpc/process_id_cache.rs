use std::collections::HashMap;

use tokio::sync::Mutex;

pub struct ProcessIdCache<T: Clone> {
    cache: Mutex<HashMap<String, T>>,
}

impl<T: Clone> ProcessIdCache<T> {
    pub fn new() -> Self {
        return Self {
            cache: Mutex::new(HashMap::new()),
        };
    }

    pub async fn get(&self, key: &str) -> Option<T> {
        let cache = self.cache.lock().await;
        return cache.get(key).map(|x| x.clone());
    }

    pub async fn set(&self, key: &str, value: T) {
        let mut cache = self.cache.lock().await;
        cache.insert(key.to_string(), value);
    }
}
