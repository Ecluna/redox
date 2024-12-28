use std::collections::{HashMap, HashSet, BTreeMap};
use std::sync::Arc;
use tokio::sync::Mutex;
use redox_protocol::RedoxValue;
use crate::persistence::Persistence;
use std::time::{SystemTime, UNIX_EPOCH};

/// 存储结构体，提供线程安全的数据存储和访问
/// 支持多种数据类型：字符串、列表、集合、哈希表和有序集合
#[derive(Clone)]
pub struct Storage {
    /// 核心数据存储，使用 Arc<Mutex> 实现线程安全
    /// HashMap 的键是字符串，值是 RedoxValue 枚举
    data: Arc<Mutex<HashMap<String, RedoxValue>>>,
    /// 持久化管理器，可选
    persistence: Option<Persistence>,
}

impl Storage {
    /// 创建新的存储实例
    /// 
    /// # Arguments
    /// * `persistence` - 可选的持久化管理器
    /// 
    /// # Returns
    /// 新的存储实例，如果提供了持久化管理器，会自动加载已保存的数据
    pub fn new(persistence: Option<Persistence>) -> Self {
        // 尝试从持久化存储加载数据
        let data = match &persistence {
            Some(p) => {
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        match p.load().await {
                            Ok(data) => data,
                            Err(e) => {
                                eprintln!("Error loading data: {}", e);
                                HashMap::new()
                            }
                        }
                    })
                })
            }
            None => HashMap::new(),
        };

        let storage = Storage {
            data: Arc::new(Mutex::new(data)),
            persistence,
        };

        // 如果启用了持久化，启动自动保存任务
        if let Some(p) = storage.persistence.clone() {
            let data = storage.data.clone();
            tokio::spawn(async move {
                p.start_auto_save(data).await;
            });
        }

        storage
    }

    /// 标记数据已修改
    fn mark_dirty(&self) {
        if let Some(p) = &self.persistence {
            p.mark_dirty();
        }
    }

    // 字符串操作
    /// 设置字符串值
    /// 
    /// # Arguments
    /// * `key` - 键
    /// * `value` - 值
    pub async fn set_string(&self, key: String, value: String) {
        let mut data = self.data.lock().await;
        data.insert(key, RedoxValue::String(value));
        self.mark_dirty();
    }

    /// 获取字符串值
    /// 
    /// # Arguments
    /// * `key` - 键
    /// 
    /// # Returns
    /// * `Some(String)` - 找到的值
    /// * `None` - 键不存在或类型不匹配
    async fn get_if_not_expired(&self, key: &str) -> Option<RedoxValue> {
        if self.check_expired(key).await {
            return None;
        }
        let data = self.data.lock().await;
        data.get(key).cloned()
    }

    pub async fn get_string(&self, key: &str) -> Option<String> {
        match self.get_if_not_expired(key).await {
            Some(RedoxValue::String(s)) => Some(s),
            _ => None,
        }
    }

    // 列表操作
    /// 在列表左端插入元素
    /// 
    /// # Arguments
    /// * `key` - 列表的键
    /// * `value` - 要插入的值
    /// 
    /// # Returns
    /// 操作后列表的长度
    pub async fn lpush(&self, key: String, value: String) -> usize {
        let mut data = self.data.lock().await;
        let result = match data.get_mut(&key) {
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
            _ => 0
        };
        if result > 0 {
            self.mark_dirty();
        }
        result
    }

    pub async fn rpush(&self, key: String, value: String) -> usize {
        let mut data = self.data.lock().await;
        let result = match data.get_mut(&key) {
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
        };
        if result > 0 {
            self.mark_dirty();
        }
        result
    }

    pub async fn lpop(&self, key: &str) -> Option<String> {
        let mut data = self.data.lock().await;
        let result = match data.get_mut(key) {
            Some(RedoxValue::List(list)) => {
                if list.is_empty() {
                    None
                } else {
                    Some(list.remove(0))
                }
            }
            _ => None,
        };
        if result.is_some() {
            self.mark_dirty();
        }
        result
    }

    pub async fn rpop(&self, key: &str) -> Option<String> {
        let mut data = self.data.lock().await;
        let result = match data.get_mut(key) {
            Some(RedoxValue::List(list)) => list.pop(),
            _ => None,
        };
        if result.is_some() {
            self.mark_dirty();
        }
        result
    }

    pub async fn lrange(&self, key: &str, start: i64, stop: i64) -> Option<Vec<String>> {
        match self.get_if_not_expired(key).await {
            Some(RedoxValue::List(list)) => {
                let len = list.len() as i64;
                let (start, stop) = normalize_range(start, stop, len);
                Some(list[start..=stop].to_vec())
            }
            _ => None,
        }
    }

    // 集合操作
    /// 向集合添加成员
    /// 
    /// # Arguments
    /// * `key` - 集合的键
    /// * `member` - 要添加的成员
    /// 
    /// # Returns
    /// * `true` - 添加成功（成员是新的）
    /// * `false` - 添加失败（成员已存在或类型错误）
    pub async fn sadd(&self, key: String, member: String) -> bool {
        let mut data = self.data.lock().await;
        let result = match data.get_mut(&key) {
            Some(RedoxValue::Set(set)) => set.insert(member),
            None => {
                let mut set = HashSet::new();
                let result = set.insert(member);
                data.insert(key, RedoxValue::Set(set));
                result
            }
            _ => false
        };
        if result {
            self.mark_dirty();
        }
        result
    }

    pub async fn srem(&self, key: &str, member: &str) -> bool {
        let mut data = self.data.lock().await;
        let result = match data.get_mut(key) {
            Some(RedoxValue::Set(set)) => set.remove(member),
            _ => false,
        };
        if result {
            self.mark_dirty();
        }
        result
    }

    pub async fn smembers(&self, key: &str) -> Option<Vec<String>> {
        match self.get_if_not_expired(key).await {
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
    /// 设置哈希表字段的值
    /// 
    /// # Arguments
    /// * `key` - 哈希表的键
    /// * `field` - 字段名
    /// * `value` - 字段值
    /// 
    /// # Returns
    /// * `true` - 设置了新字段
    /// * `false` - 更新了已存在的字段
    pub async fn hset(&self, key: String, field: String, value: String) -> bool {
        let mut data = self.data.lock().await;
        let result = match data.get_mut(&key) {
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
            _ => false
        };
        if result {
            self.mark_dirty();
        }
        result
    }

    pub async fn hget(&self, key: &str, field: &str) -> Option<String> {
        match self.get_if_not_expired(key).await {
            Some(RedoxValue::Hash(hash)) => hash.get(field).cloned(),
            _ => None,
        }
    }

    pub async fn hdel(&self, key: &str, field: &str) -> bool {
        let mut data = self.data.lock().await;
        let result = match data.get_mut(key) {
            Some(RedoxValue::Hash(hash)) => hash.remove(field).is_some(),
            _ => false,
        };
        if result {
            self.mark_dirty();
        }
        result
    }

    pub async fn hgetall(&self, key: &str) -> Option<HashMap<String, String>> {
        let data = self.data.lock().await;
        match data.get(key) {
            Some(RedoxValue::Hash(hash)) => Some(hash.clone()),
            _ => None,
        }
    }

    // 有序集合操作
    /// 向有序集合添加成员
    /// 
    /// # Arguments
    /// * `key` - 有序集合的键
    /// * `score` - 成员的分数
    /// * `member` - 成员名
    /// 
    /// # Returns
    /// * `true` - 添加了新成员
    /// * `false` - 更新了已存在的成员
    pub async fn zadd(&self, key: String, score: f64, member: String) -> bool {
        let mut data = self.data.lock().await;
        let result = match data.get_mut(&key) {
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
            _ => false
        };
        if result {
            self.mark_dirty();
        }
        result
    }

    pub async fn zrem(&self, key: &str, member: &str) -> bool {
        let mut data = self.data.lock().await;
        let result = match data.get_mut(key) {
            Some(RedoxValue::SortedSet(zset)) => zset.remove(member).is_some(),
            _ => false,
        };
        if result {
            self.mark_dirty();
        }
        result
    }

    pub async fn zrange(&self, key: &str, start: i64, stop: i64) -> Option<Vec<(String, f64)>> {
        match self.get_if_not_expired(key).await {
            Some(RedoxValue::SortedSet(zset)) => {
                let len = zset.len() as i64;
                if len == 0 {
                    return Some(vec![]);
                }
                
                // 按分数升序排序
                let mut members: Vec<(String, f64)> = zset.iter()
                    .map(|(k, v)| (k.clone(), *v))
                    .collect();
                members.sort_by(|a, b| {
                    a.1.partial_cmp(&b.1)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then(a.0.cmp(&b.0))
                });
                
                let (start, stop) = normalize_range(start, stop, len);
                Some(members.into_iter()
                    .skip(start)
                    .take(stop - start + 1)
                    .collect())
            }
            _ => None,
        }
    }

    pub async fn zrangebyscore(&self, key: &str, min: f64, max: f64) -> Option<Vec<(String, f64)>> {
        let data = self.data.lock().await;
        match data.get(key) {
            Some(RedoxValue::SortedSet(zset)) => {
                // 先按分数排序，分数同时按成员字典序排序
                let mut members: Vec<(String, f64)> = zset.iter()
                    .map(|(k, v)| (k.clone(), *v))
                    .collect();
                members.sort_by(|a, b| {
                    a.1.partial_cmp(&b.1)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then(a.0.cmp(&b.0))
                });
                
                Some(members.into_iter()
                    .filter(|(_, score)| *score >= min && *score <= max)
                    .collect())
            }
            _ => None,
        }
    }

    /// 批量设置字符串值
    pub async fn mset(&self, pairs: Vec<(String, String)>) -> usize {
        let mut data = self.data.lock().await;
        let mut count = 0;
        for (key, value) in pairs {
            data.insert(key, RedoxValue::String(value));
            count += 1;
        }
        if count > 0 {
            self.mark_dirty();
        }
        count
    }

    /// 批量获取字符串值
    pub async fn mget(&self, keys: &[String]) -> Vec<Option<String>> {
        let data = self.data.lock().await;
        keys.iter().map(|key| {
            match data.get(key) {
                Some(RedoxValue::String(s)) => Some(s.clone()),
                _ => None,
            }
        }).collect()
    }

    /// 设置键的过期时间（秒）
    pub async fn expire(&self, key: &str, seconds: u64) -> bool {
        let data = self.data.lock().await;
        if !data.contains_key(key) {
            return false;
        }
        
        let expires = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + seconds;
            
        if let Some(p) = &self.persistence {
            p.set_expiry(key.to_string(), expires).await;
            self.mark_dirty();
        }
        true
    }

    /// 检查键是否过期
    async fn is_expired(&self, key: &str) -> bool {
        if let Some(p) = &self.persistence {
            if let Some(expires) = p.get_expiry(key).await {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                return now >= expires;
            }
        }
        false
    }

    /// 清理过期的键
    pub async fn cleanup_expired(&self) {
        let mut data = self.data.lock().await;
        let mut expired_keys = Vec::new();
        
        // 收集过期键
        for key in data.keys() {
            if self.is_expired(key).await {
                expired_keys.push(key.clone());
            }
        }
        
        // 删除过期的键
        if !expired_keys.is_empty() {
            for key in expired_keys {
                data.remove(&key);
            }
            self.mark_dirty();
        }
    }

    /// 获取存储统计信息
    pub async fn info(&self) -> HashMap<String, String> {
        let data = self.data.lock().await;
        let mut info = HashMap::new();
        
        // 先统计所有类型的键数量
        let mut strings = 0;
        let mut lists = 0;
        let mut sets = 0;
        let mut hashes = 0;
        let mut zsets = 0;
        
        for value in data.values() {
            match value {
                RedoxValue::String(_) => strings += 1,
                RedoxValue::List(_) => lists += 1,
                RedoxValue::Set(_) => sets += 1,
                RedoxValue::Hash(_) => hashes += 1,
                RedoxValue::SortedSet(_) => zsets += 1,
            }
        }
        
        // 按字母顺序插入统计信息
        info.insert("hashes".to_string(), hashes.to_string());
        info.insert("keys".to_string(), data.len().to_string());
        info.insert("lists".to_string(), lists.to_string());
        info.insert("sets".to_string(), sets.to_string());
        info.insert("strings".to_string(), strings.to_string());
        info.insert("zsets".to_string(), zsets.to_string());
        
        info
    }

    /// 删除一个或多个键
    /// 返回实际删除的键的数量
    pub async fn del(&self, keys: &[String]) -> usize {
        let mut data = self.data.lock().await;
        let mut count = 0;
        
        for key in keys {
            if data.remove(key).is_some() {
                count += 1;
            }
        }
        
        if count > 0 {
            self.mark_dirty();
        }
        
        count
    }

    pub async fn ttl(&self, key: &str) -> Option<i64> {
        if let Some(p) = &self.persistence {
            if let Some(expires) = p.get_expiry(key).await {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                if now >= expires {
                    return Some(-1);  // 已过期
                }
                return Some((expires - now) as i64);
            }
        }
        None  // 键不存在或没有设置过期时间
    }
    
    pub async fn persist(&self, key: &str) -> bool {
        if let Some(p) = &self.persistence {
            if p.remove_expiry(key).await {
                self.mark_dirty();
                return true;
            }
        }
        false
    }

    // 在每次访问键时检查过期
    async fn check_expired(&self, key: &str) -> bool {
        if self.is_expired(key).await {
            let mut data = self.data.lock().await;
            data.remove(key);
            self.mark_dirty();
            true
        } else {
            false
        }
    }

    // 添加定期清理任务
    pub async fn start_cleanup_task(self) {
        let interval = tokio::time::Duration::from_secs(10); // 每10秒清理一次
        let mut interval = tokio::time::interval(interval);
        
        loop {
            interval.tick().await;
            self.cleanup_expired().await;
        }
    }
}

/// 规范化范围索引
/// 
/// # Arguments
/// * `start` - 起始索引（可以是负数）
/// * `stop` - 结束索引（可以是负数）
/// * `len` - 列表总长度
/// 
/// # Returns
/// (start, stop) 转换后的索引对，确保在有效范围内
fn normalize_range(start: i64, stop: i64, len: i64) -> (usize, usize) {
    // 处理负数索引
    let start = if start < 0 { (len + start).max(0) } else { start.min(len - 1) };
    let stop = if stop < 0 { (len + stop).max(0) } else { stop.min(len - 1) };
    
    // 确保 start 不大于 stop
    let start = start.min(stop);
    let stop = stop.max(start);
    
    // 转换为 usize
    (start as usize, stop as usize)
}