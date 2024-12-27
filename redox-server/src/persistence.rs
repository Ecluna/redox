use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter};
use std::path::Path;
use std::time::Duration;
use tokio::time;
use redox_protocol::RedoxValue;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct PersistentData {
    data: HashMap<String, RedoxValue>,
}

pub struct Persistence {
    file_path: String,
    save_interval: Duration,
}

impl Persistence {
    pub fn new(file_path: String, save_interval: Duration) -> Self {
        Self {
            file_path,
            save_interval,
        }
    }

    /// 加载数据
    pub fn load(&self) -> io::Result<HashMap<String, RedoxValue>> {
        if !Path::new(&self.file_path).exists() {
            return Ok(HashMap::new());
        }

        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let data: PersistentData = serde_json::from_reader(reader)?;
        Ok(data.data)
    }

    /// 保存数据
    pub fn save(&self, data: &HashMap<String, RedoxValue>) -> io::Result<()> {
        let persistent_data = PersistentData {
            data: data.clone(),
        };

        // 创建临时文件
        let temp_path = format!("{}.temp", self.file_path);
        let file = File::create(&temp_path)?;
        let writer = BufWriter::new(file);

        // 写入数据
        serde_json::to_writer(writer, &persistent_data)?;

        // 原子性地替换文件
        fs::rename(temp_path, &self.file_path)?;
        Ok(())
    }

    /// 启动自动保存任务
    pub async fn start_auto_save(
        self,
        data: std::sync::Arc<tokio::sync::Mutex<HashMap<String, RedoxValue>>>,
    ) {
        tokio::spawn(async move {
            let mut interval = time::interval(self.save_interval);
            loop {
                interval.tick().await;
                let data = data.lock().await;
                if let Err(e) = self.save(&data) {
                    eprintln!("Error saving data: {}", e);
                }
            }
        });
    }
} 