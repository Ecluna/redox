use std::collections::{HashMap, HashSet, BTreeMap};
use std::sync::Arc;
use tokio::sync::Mutex;
use redox_protocol::RedoxValue;

/// 存储结构体，使用 Arc<Mutex> 实现线程安全的共享存储
#[derive(Clone)]
pub struct Storage {
    data: Arc<Mutex<HashMap<String, RedoxValue>>>,
}

impl Storage {
    /// 创建新的存储实例
    pub fn new() -> Self {
        Storage {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // 字符串操作
    pub async fn set_string(&self, key: String, value: String) {
        let mut data = self.data.lock().await;
        data.insert(key, RedoxValue::String(value));
    }

    pub async fn get_string(&self, key: &str) -> Option<String> {
        let data = self.data.lock().await;
        match data.get(key) {
            Some(RedoxValue::String(s)) => Some(s.clone()),
            _ => None,
        }
    }

    // 列表操作
    pub async fn lpush(&self, key: String, value: String) -> usize {
        let mut data = self.data.lock().await;
        match data.get_mut(&key) {
            Some(RedoxValue::List(list)) => {
                list.insert(0, value);
                list.len()
            }
            None => {
                let mut list = Vec::new();
                list.push(value);
                data.insert(key, RedoxValue::List(list));
                1
            }
            _ => {
                // 如果键存在但类型不是列表，返回0
                0
            }
        }
    }

    pub async fn rpush(&self, key: String, value: String) -> usize {
        let mut data = self.data.lock().await;
        match data.get_mut(&key) {
            Some(RedoxValue::List(list)) => {
                list.push(value);
                list.len()
            }
            None => {
                let mut list = Vec::new();
                list.push(value);
                data.insert(key, RedoxValue::List(list));
                1
            }
            _ => 0
        }
    }

    pub async fn lpop(&self, key: &str) -> Option<String> {
        let mut data = self.data.lock().await;
        match data.get_mut(key) {
            Some(RedoxValue::List(list)) => list.pop(),
            _ => None,
        }
    }

    pub async fn rpop(&self, key: &str) -> Option<String> {
        let mut data = self.data.lock().await;
        match data.get_mut(key) {
            Some(RedoxValue::List(list)) => {
                if list.is_empty() {
                    None
                } else {
                    Some(list.remove(0))
                }
            }
            _ => None,
        }
    }

    pub async fn lrange(&self, key: &str, start: i64, stop: i64) -> Option<Vec<String>> {
        let data = self.data.lock().await;
        match data.get(key) {
            Some(RedoxValue::List(list)) => {
                let len = list.len() as i64;
                let (start, stop) = normalize_range(start, stop, len);
                Some(list[start..=stop].to_vec())
            }
            _ => None,
        }
    }

    // 集合操作
    pub async fn sadd(&self, key: String, member: String) -> bool {
        let mut data = self.data.lock().await;
        match data.get_mut(&key) {
            Some(RedoxValue::Set(set)) => set.insert(member),
            None => {
                let mut set = HashSet::new();
                let result = set.insert(member);
                data.insert(key, RedoxValue::Set(set));
                result
            }
            _ => false,
        }
    }

    pub async fn srem(&self, key: &str, member: &str) -> bool {
        let mut data = self.data.lock().await;
        match data.get_mut(key) {
            Some(RedoxValue::Set(set)) => set.remove(member),
            _ => false,
        }
    }

    pub async fn smembers(&self, key: &str) -> Option<Vec<String>> {
        let data = self.data.lock().await;
        match data.get(key) {
            Some(RedoxValue::Set(set)) => Some(set.iter().cloned().collect()),
            _ => None,
        }
    }

    pub async fn sismember(&self, key: &str, member: &str) -> bool {
        let data = self.data.lock().await;
        match data.get(key) {
            Some(RedoxValue::Set(set)) => set.contains(member),
            _ => false,
        }
    }

    // 哈希表操作
    pub async fn hset(&self, key: String, field: String, value: String) -> bool {
        let mut data = self.data.lock().await;
        match data.get_mut(&key) {
            Some(RedoxValue::Hash(hash)) => {
                let is_new = !hash.contains_key(&field);
                hash.insert(field, value);
                is_new
            }
            None => {
                let mut hash = HashMap::new();
                hash.insert(field, value);
                data.insert(key, RedoxValue::Hash(hash));
                true
            }
            _ => false,
        }
    }

    pub async fn hget(&self, key: &str, field: &str) -> Option<String> {
        let data = self.data.lock().await;
        match data.get(key) {
            Some(RedoxValue::Hash(hash)) => hash.get(field).cloned(),
            _ => None,
        }
    }

    pub async fn hdel(&self, key: &str, field: &str) -> bool {
        let mut data = self.data.lock().await;
        match data.get_mut(key) {
            Some(RedoxValue::Hash(hash)) => hash.remove(field).is_some(),
            _ => false,
        }
    }

    pub async fn hgetall(&self, key: &str) -> Option<HashMap<String, String>> {
        let data = self.data.lock().await;
        match data.get(key) {
            Some(RedoxValue::Hash(hash)) => Some(hash.clone()),
            _ => None,
        }
    }

    // 有序集合操作
    pub async fn zadd(&self, key: String, score: f64, member: String) -> bool {
        let mut data = self.data.lock().await;
        match data.get_mut(&key) {
            Some(RedoxValue::SortedSet(zset)) => {
                let is_new = !zset.contains_key(&member);
                zset.insert(member, score);
                is_new
            }
            None => {
                let mut zset = BTreeMap::new();
                zset.insert(member, score);
                data.insert(key, RedoxValue::SortedSet(zset));
                true
            }
            _ => false,
        }
    }

    pub async fn zrem(&self, key: &str, member: &str) -> bool {
        let mut data = self.data.lock().await;
        match data.get_mut(key) {
            Some(RedoxValue::SortedSet(zset)) => zset.remove(member).is_some(),
            _ => false,
        }
    }

    pub async fn zrange(&self, key: &str, start: i64, stop: i64) -> Option<Vec<(String, f64)>> {
        let data = self.data.lock().await;
        match data.get(key) {
            Some(RedoxValue::SortedSet(zset)) => {
                let len = zset.len() as i64;
                let (start, stop) = normalize_range(start, stop, len);
                Some(zset.iter()
                    .skip(start)
                    .take(stop - start + 1)
                    .map(|(k, v)| (k.clone(), *v))
                    .collect())
            }
            _ => None,
        }
    }

    pub async fn zrangebyscore(&self, key: &str, min: f64, max: f64) -> Option<Vec<(String, f64)>> {
        let data = self.data.lock().await;
        match data.get(key) {
            Some(RedoxValue::SortedSet(zset)) => {
                Some(zset.iter()
                    .filter(|(_, score)| **score >= min && **score <= max)
                    .map(|(k, v)| (k.clone(), *v))
                    .collect())
            }
            _ => None,
        }
    }
}

// 辅助函数：规范化范围索引
fn normalize_range(start: i64, stop: i64, len: i64) -> (usize, usize) {
    let start = if start < 0 { len + start } else { start };
    let stop = if stop < 0 { len + stop } else { stop };
    let start = start.max(0) as usize;
    let stop = stop.min(len - 1) as usize;
    (start, stop)
}