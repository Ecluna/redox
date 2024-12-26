use crate::storage::Storage;
use redox_protocol::{Command, Protocol, Response};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

/// 服务器结构体，持有存储实例
pub struct Server {
    storage: Storage,
}

impl Server {
    /// 创建新的服务器实例
    pub fn new(storage: Storage) -> Self {
        Server { storage }
    }

    /// 运行服务器，监听指定地址
    pub async fn run(&self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 绑定 TCP 监听器
        let listener = TcpListener::bind(addr).await?;
        println!("Server listening on {}", addr);

        // 循环接受新的连接
        loop {
            let (socket, _) = listener.accept().await?;
            let storage = self.storage.clone();
            
            // 为每个连接创建新的任务
            tokio::spawn(async move {
                if let Err(e) = handle_connection(socket, storage).await {
                    eprintln!("Error handling connection: {}", e);
                }
            });
        }
    }
}

/// 处理单个客户端连接
async fn handle_connection(
    mut socket: TcpStream,
    storage: Storage,
) -> Result<(), Box<dyn std::error::Error>> {
    // 分离读写流
    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    // 处理命令循环
    loop {
        line.clear();
        // 读取一行数据，如果连接关闭则退出
        if reader.read_line(&mut line).await? == 0 {
            break;
        }

        // 解析命令
        let cmd = match Protocol::decode_command(&line) {
            Ok(cmd) => cmd,
            Err(e) => {
                // 发送错误响应
                let response = Protocol::encode_response(&Response::Error(e));
                writer.write_all(response.as_bytes()).await?;
                continue;
            }
        };

        // 处理命令并生成响应
        let response = match cmd {
            Command::Set { key, value } => {
                storage.set(key, value).await;
                Response::Ok
            }
            Command::Get { key } => {
                match storage.get(&key).await {
                    Some(value) => Response::Value(value),
                    None => Response::Value("NIL".to_string()),
                }
            }
        };

        // 发送响应
        let response_str = Protocol::encode_response(&response);
        writer.write_all(response_str.as_bytes()).await?;
    }

    Ok(())
} 