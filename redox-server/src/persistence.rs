use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time;
use tokio::sync::Mutex;
use redox_protocol::RedoxValue;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::fs::File as TokioFile;
use tokio::io::{self as tokio_io, AsyncReadExt, AsyncWriteExt, BufReader as TokioBufReader, BufWriter as TokioBufWriter};

/// 持久化数据的序列化结构
/// 使用 serde 进行 JSON 序列化和反序列化
#[derive(Serialize, Deserialize)]
struct PersistentData {
    /// 存储所有键值对的哈希表
    data: HashMap<String, RedoxValue>,
    expiry: HashMap<String, u64>,
}

/// 旧版本的数据格式
#[derive(Serialize, Deserialize)]
struct LegacyData {
    data: HashMap<String, RedoxValue>,
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

    /// 从文件载数据
    /// 
    /// # Returns
    /// * `Ok(HashMap)` - 成功加载的数据
    /// * `Err` - 加载过程中的错误
    pub async fn load(&self) -> tokio_io::Result<HashMap<String, RedoxValue>> {
        if !Path::new(&self.file_path).exists() {
            eprintln!("Data file not found: {}", self.file_path);
            return Ok(HashMap::new());
        }

        let file = match TokioFile::open(&self.file_path).await {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error opening data file: {}", e);
                return Err(e);
            }
        };
        
        let mut content = String::new();
        let mut reader = TokioBufReader::new(file);
        reader.read_to_string(&mut content).await?;

        // 尝试以新格式读取
        match serde_json::from_str::<PersistentData>(&content) {
            Ok(persistent_data) => {
                let mut expiry = self.expiry.lock().await;
                *expiry = persistent_data.expiry;
                Ok(persistent_data.data)
            }
            Err(e) => {
                eprintln!("Failed to read as new format: {}", e);
                // 如果失败，尝试以旧格式读取
                match serde_json::from_str::<LegacyData>(&content) {
                    Ok(legacy_data) => {
                        println!("Successfully loaded data in legacy format");
                        Ok(legacy_data.data)
                    }
                    Err(e) => {
                        eprintln!("Error deserializing data: {}", e);
                        Err(tokio_io::Error::new(tokio_io::ErrorKind::InvalidData, e))
                    }
                }
            }
        }
    }

    /// 将数据保存到文件
    /// 
    /// # Arguments
    /// * `data` - 要保存的数据
    /// 
    /// # Returns
    /// * `Ok(())` - 保存成功
    /// * `Err` - 保存过程中的错误
    pub async fn save(&self, data: &HashMap<String, RedoxValue>) -> tokio_io::Result<()> {
        let expiry = self.expiry.lock().await;
        let persistent_data = PersistentData {
            data: data.clone(),
            expiry: expiry.clone(),
        };

        let temp_path = format!("{}.temp", self.file_path);
        let file = TokioFile::create(&temp_path).await?;
        let mut writer = TokioBufWriter::new(file);

        let json = serde_json::to_string(&persistent_data)?;
        writer.write_all(json.as_bytes()).await?;
        writer.flush().await?;

        tokio::fs::rename(temp_path, &self.file_path).await?;
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
            
            if !self.dirty.load(Ordering::Relaxed) {
                continue;
            }

            let data = data.lock().await;
            if let Err(e) = self.save(&data).await {
                eprintln!("Error saving data: {}", e);
            } else {
                self.dirty.store(false, Ordering::Relaxed);
                self.last_save.store(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    Ordering::Relaxed
                );
            }
        }
    }

    #[allow(dead_code)]
    pub async fn set_expiry(&self, key: String, expires: u64) {
        let mut expiry = self.expiry.lock().await;
        expiry.insert(key, expires);
    }

    #[allow(dead_code)]
    pub async fn get_expiry(&self, key: &str) -> Option<u64> {
        let expiry = self.expiry.lock().await;
        expiry.get(key).copied()
    }

    pub fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }
} 