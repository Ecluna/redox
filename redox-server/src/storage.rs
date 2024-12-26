use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Storage {
    data: Arc<Mutex<HashMap<String, String>>>,
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn set(&self, key: String, value: String) {
        let mut data = self.data.lock().await;
        data.insert(key, value);
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let data = self.data.lock().await;
        data.get(key).cloned()
    }
} 