use crate::storage::Storage;
use redox_protocol::{Command, Protocol, Response};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use std::io;
use std::sync::Arc;

/// 服务器结构体，持有存储实例
pub struct Server {
    storage: Storage,
    password: Option<Arc<String>>,
}

impl Server {
    /// 创建新的服务器实例
    pub fn new(storage: Storage, password: Option<String>) -> Self {
        Server { 
            storage,
            password: password.map(Arc::new),
        }
    }

    /// 尝试绑定地址，返回是否成功
    pub async fn try_bind(&self, addr: &str) -> io::Result<()> {
        TcpListener::bind(addr).await?;
        Ok(())
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
            let password = self.password.clone();
            
            // 为每个连接创建新的任务
            tokio::spawn(async move {
                if let Err(e) = handle_connection(socket, storage, password).await {
                    eprintln!("Error handling connection: {}", e);
                }
            });
        }
    }
}

/// 处理单个客户端连接的状态
struct ConnectionState {
    authenticated: bool,
    requires_auth: bool,
}

/// 处理单个客户端连接
async fn handle_connection(
    mut socket: TcpStream,
    storage: Storage,
    password: Option<Arc<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    let mut state = ConnectionState {
        authenticated: password.is_none(), // 如果没有设置密码，则默认已认证
        requires_auth: password.is_some(), // 是否需要认证
    };

    loop {
        line.clear();
        if reader.read_line(&mut line).await? == 0 {
            break;
        }

        let cmd = match Protocol::decode_command(&line) {
            Ok(cmd) => cmd,
            Err(e) => {
                let response = Protocol::encode_response(&Response::Error(e));
                writer.write_all(response.as_bytes()).await?;
                continue;
            }
        };

        let response = match cmd {
            Command::Auth { password: input_password } => {
                if !state.requires_auth {
                    Response::Error("Authentication not required".to_string())
                } else if let Some(correct_password) = &password {
                    if input_password == **correct_password {
                        state.authenticated = true;
                        Response::Ok
                    } else {
                        Response::Error("Invalid password".to_string())
                    }
                } else {
                    Response::Error("Server error".to_string())
                }
            }
            _ if !state.authenticated => {
                Response::Error("Authentication required".to_string())
            }
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

        let response_str = Protocol::encode_response(&response);
        writer.write_all(response_str.as_bytes()).await?;
    }

    Ok(())
} 