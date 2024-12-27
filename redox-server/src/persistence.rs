use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter};
use std::path::Path;
use std::time::Duration;
use tokio::time;
use tokio::sync::Mutex;
use redox_protocol::RedoxValue;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// 持久化数据的序列化结构
/// 使用 serde 进行 JSON 序列化和反序列化
#[derive(Serialize, Deserialize)]
struct PersistentData {
    /// 存储所有键值对的哈希表
    data: HashMap<String, RedoxValue>,
    expiry: HashMap<String, u64>,
}

/// 持久化管理器
/// 负责数据的加载、保存和自动保存
#[derive(Clone)]
pub struct Persistence {
    /// 数据文件的路径
    file_path: String,
    /// 自动保存的时间间隔
    save_interval: Duration,
    /// 上次保存时间
    last_save: Arc<AtomicU64>,
    /// 脏标记
    dirty: Arc<AtomicBool>,
    expiry: Arc<Mutex<HashMap<String, u64>>>,
}

impl Persistence {
    /// 创建新的持久化管理器实例
    /// 
    /// # Arguments
    /// * `file_path` - 数据文件的路径
    /// * `save_interval` - 自动保存的时间间隔
    pub fn new(file_path: String, save_interval: Duration) -> Self {
        Self {
            file_path,
            save_interval,
            last_save: Arc::new(AtomicU64::new(0)),
            dirty: Arc::new(AtomicBool::new(false)),
            expiry: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 从文件加载数据
    /// 
    /// # Returns
    /// * `Ok(HashMap)` - 成功加载的数据
    /// * `Err` - 加载过程中的错误
    pub fn load(&self) -> io::Result<HashMap<String, RedoxValue>> {
        // 如果文件不存在，返回空哈希表
        if !Path::new(&self.file_path).exists() {
            return Ok(HashMap::new());
        }

        // 打开文件并创建带缓冲的读取器
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        
        // 从 JSON 反序列化数据
        let data: PersistentData = serde_json::from_reader(reader)?;
        
        // 加载过期时间
        let mut expiry = self.expiry.blocking_lock();
        *expiry = data.expiry;
        
        Ok(data.data)
    }

    /// 将数据保存到文件
    /// 
    /// # Arguments
    /// * `data` - 要保存的数据
    /// 
    /// # Returns
    /// * `Ok(())` - 保存成功
    /// * `Err` - 保存过程中的错误
    pub fn save(&self, data: &HashMap<String, RedoxValue>) -> io::Result<()> {
        let expiry = self.expiry.blocking_lock();
        let persistent_data = PersistentData {
            data: data.clone(),
            expiry: expiry.clone(),
        };

        // 创建临时文件，确保写入操作的原子性
        let temp_path = format!("{}.temp", self.file_path);
        let file = File::create(&temp_path)?;
        let writer = BufWriter::new(file);

        // 将数据序列化为 JSON 并写入临时文件
        serde_json::to_writer(writer, &persistent_data)?;

        // 原子性地用临时文件替换原文件
        fs::rename(temp_path, &self.file_path)?;
        Ok(())
    }

    /// 启动自动保存任务
    /// 
    /// # Arguments
    /// * `data` - 要保存的数据的共享引用
    /// 
    /// 这个方法会创建一个新的异步任务，定期保存数据
    pub async fn start_auto_save(
        self,
        data: Arc<tokio::sync::Mutex<HashMap<String, RedoxValue>>>,
    ) {
        let mut interval = time::interval(self.save_interval);
        loop {
            interval.tick().await;
            
            // 检查是否需要保存
            if !self.dirty.load(Ordering::Relaxed) {
                continue;
            }

            let data = data.lock().await;
            if let Err(e) = self.save(&data) {
                eprintln!("Error saving data: {}", e);
            } else {
                self.dirty.store(false, Ordering::Relaxed);
                self.last_save.store(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    Ordering::Relaxed
                );
            }
        }
    }

    pub async fn set_expiry(&self, key: String, expires: u64) {
        let mut expiry = self.expiry.lock().await;
        expiry.insert(key, expires);
    }

    pub async fn get_expiry(&self, key: &str) -> Option<u64> {
        let expiry = self.expiry.lock().await;
        expiry.get(key).copied()
    }

    pub fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }
} 