use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 存储结构体，使用 Arc<Mutex> 实现线程安全的共享存储
#[derive(Clone)]
pub struct Storage {
    // Arc 提供多线程共享，Mutex 确保数据访问的互斥性
    data: Arc<Mutex<HashMap<String, String>>>,
}

impl Storage {
    /// 创建新的存储实例
    pub fn new() -> Self {
        Storage {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 设置键值对
    pub async fn set(&self, key: String, value: String) {
        // 获取锁并插入数据
        let mut data = self.data.lock().await;
        data.insert(key, value);
    }

    /// 获取指定键的值
    pub async fn get(&self, key: &str) -> Option<String> {
        // 获取锁并克隆值（避免长时间持有锁）
        let data = self.data.lock().await;
        data.get(key).cloned()
    }
} 