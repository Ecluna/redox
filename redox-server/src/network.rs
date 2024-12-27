use crate::storage::Storage;
use redox_protocol::{Command, Protocol, Response, RedoxValue};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use std::io;
use std::sync::Arc;

/// 服务器结构体
/// 管理网络连接和存储实例
pub struct Server {
    /// 存储实例，用于数据操作
    storage: Storage,
    /// 可选的认证密码
    password: Option<Arc<String>>,
}

impl Server {
    /// 创建新的服务器实例
    /// 
    /// # Arguments
    /// * `storage` - 存储实例
    /// * `password` - 可选的认证密码
    pub fn new(storage: Storage, password: Option<String>) -> Self {
        Server { 
            storage,
            password: password.map(Arc::new),
        }
    }

    /// 尝试绑定到指定地址
    /// 
    /// # Arguments
    /// * `addr` - 要绑定的地址（如 "127.0.0.1:6379"）
    /// 
    /// # Returns
    /// * `Ok(())` - 绑定成功
    /// * `Err` - 绑定失败的错误
    pub async fn try_bind(&self, addr: &str) -> io::Result<()> {
        TcpListener::bind(addr).await?;
        Ok(())
    }

    /// 运行服务器，监听连接
    /// 
    /// # Arguments
    /// * `addr` - 监听地址
    /// 
    /// # Returns
    /// * `Ok(())` - 服务器正常退出
    /// * `Err` - 运行过程中的错误
    pub async fn run(&self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 绑定 TCP 监听器
        let listener = TcpListener::bind(addr).await?;
        println!("Server listening on {}", addr);

        // 循环接受新的连接
        loop {
            let (socket, _) = listener.accept().await?;
            let storage = self.storage.clone();
            let password = self.password.clone();
            
            // 为每个连接创建新的异步任务
            tokio::spawn(async move {
                if let Err(e) = handle_connection(socket, storage, password).await {
                    eprintln!("Error handling connection: {}", e);
                }
            });
        }
    }
}

/// 客户端连接的状态
struct ConnectionState {
    /// 是否已通过认证
    authenticated: bool,
}

/// 处理单个客户端连接
/// 
/// # Arguments
/// * `socket` - TCP 连接
/// * `storage` - 存储实例
/// * `password` - 可选的认证密码
/// 
/// # Returns
/// * `Ok(())` - 连接正常关闭
/// * `Err` - 处理过程中的错误
async fn handle_connection(
    mut socket: TcpStream,
    storage: Storage,
    password: Option<Arc<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    // 初始化连接状态
    let mut state = ConnectionState {  // 添加 mut
        authenticated: password.is_none(),  // 如果没有设置密码，则默认已认证
    };

    // 主处理循环
    loop {
        line.clear();
        if reader.read_line(&mut line).await? == 0 {
            break;  // 连接关闭
        }

        // 解析命令
        let cmd = match Protocol::decode_command(&line) {
            Ok(cmd) => cmd,
            Err(e) => {
                let response = Protocol::encode_response(&Response::Error(e));
                writer.write_all(response.as_bytes()).await?;
                continue;
            }
        };

        // 处理命令并生成响应
        let response = match cmd {
            Command::Auth { password: input_password } => {
                if let Some(server_password) = &password {
                    if input_password == **server_password {
                        state.authenticated = true;  // 更新认证状态
                        Response::Ok
                    } else {
                        Response::Error("Invalid password".to_string())
                    }
                } else {
                    Response::Error("Authentication not required".to_string())
                }
            }
            _ if !state.authenticated => {
                Response::Error("Authentication required".to_string())
            }
            // 字符串操作
            Command::Set { key, value } => {
                storage.set_string(key, value).await;
                Response::Ok
            }
            Command::Get { key } => {
                match storage.get_string(&key).await {
                    Some(value) => Response::Value(RedoxValue::String(value)),
                    None => Response::Value(RedoxValue::String("NIL".to_string())),
                }
            }
            // 列表操作
            Command::LPush { key, value } => {
                let len = storage.lpush(key, value).await;
                Response::Value(RedoxValue::String(len.to_string()))
            }
            Command::RPush { key, value } => {
                let len = storage.rpush(key, value).await;
                Response::Value(RedoxValue::String(len.to_string()))
            }
            Command::LPop { key } => {
                match storage.lpop(&key).await {
                    Some(value) => Response::Value(RedoxValue::String(value)),
                    None => Response::Value(RedoxValue::String("NIL".to_string())),
                }
            }
            Command::RPop { key } => {
                match storage.rpop(&key).await {
                    Some(value) => Response::Value(RedoxValue::String(value)),
                    None => Response::Value(RedoxValue::String("NIL".to_string())),
                }
            }
            Command::LRange { key, start, stop } => {
                match storage.lrange(&key, start, stop).await {
                    Some(list) => Response::Value(RedoxValue::List(list)),
                    None => Response::Value(RedoxValue::List(vec![])),
                }
            }
            // 集合操作
            Command::SAdd { key, member } => {
                let added = storage.sadd(key, member).await;
                Response::Value(RedoxValue::String(if added { "1" } else { "0" }.to_string()))
            }
            Command::SRem { key, member } => {
                let removed = storage.srem(&key, &member).await;
                Response::Value(RedoxValue::String(if removed { "1" } else { "0" }.to_string()))
            }
            Command::SMembers { key } => {
                match storage.smembers(&key).await {
                    Some(members) => Response::Value(RedoxValue::Set(members.into_iter().collect())),
                    None => Response::Value(RedoxValue::Set(std::collections::HashSet::new())),
                }
            }
            Command::SIsMember { key, member } => {
                let is_member = storage.sismember(&key, &member).await;
                Response::Value(RedoxValue::String(if is_member { "1" } else { "0" }.to_string()))
            }
            // 哈希表操作
            Command::HSet { key, field, value } => {
                let is_new = storage.hset(key, field, value).await;
                Response::Value(RedoxValue::String(if is_new { "1" } else { "0" }.to_string()))
            }
            Command::HGet { key, field } => {
                match storage.hget(&key, &field).await {
                    Some(value) => Response::Value(RedoxValue::String(value)),
                    None => Response::Value(RedoxValue::String("NIL".to_string())),
                }
            }
            Command::HDel { key, field } => {
                let deleted = storage.hdel(&key, &field).await;
                Response::Value(RedoxValue::String(if deleted { "1" } else { "0" }.to_string()))
            }
            Command::HGetAll { key } => {
                match storage.hgetall(&key).await {
                    Some(hash) => Response::Value(RedoxValue::Hash(hash)),
                    None => Response::Value(RedoxValue::Hash(std::collections::HashMap::new())),
                }
            }
            // 有序集合操作
            Command::ZAdd { key, score, member } => {
                let added = storage.zadd(key, score, member).await;
                Response::Value(RedoxValue::String(if added { "1" } else { "0" }.to_string()))
            }
            Command::ZRem { key, member } => {
                let removed = storage.zrem(&key, &member).await;
                Response::Value(RedoxValue::String(if removed { "1" } else { "0" }.to_string()))
            }
            Command::ZRange { key, start, stop } => {
                match storage.zrange(&key, start, stop).await {
                    Some(members) => {
                        let zset = members.into_iter()
                            .map(|(member, score)| (member, score))
                            .collect();
                        Response::Value(RedoxValue::SortedSet(zset))
                    }
                    None => Response::Value(RedoxValue::SortedSet(std::collections::BTreeMap::new())),
                }
            }
            Command::ZRangeByScore { key, min, max } => {
                match storage.zrangebyscore(&key, min, max).await {
                    Some(members) => {
                        let zset = members.into_iter()
                            .map(|(member, score)| (member, score))
                            .collect();
                        Response::Value(RedoxValue::SortedSet(zset))
                    }
                    None => Response::Value(RedoxValue::SortedSet(std::collections::BTreeMap::new())),
                }
            }
            Command::MSet(pairs) => {
                let count = storage.mset(pairs).await;
                Response::Integer(count)
            }
            Command::MGet(keys) => {
                let values = storage.mget(&keys).await;
                Response::Array(values)
            }
            Command::Info => {
                let info = storage.info().await;
                Response::Info(info)
            }
            Command::Del(keys) => {
                let count = storage.del(&keys).await;
                Response::Integer(count)
            }
        };

        // 发送响应
        let response_str = Protocol::encode_response(&response);
        writer.write_all(response_str.as_bytes()).await?;
    }

    Ok(())
}

async fn write_response(socket: &mut TcpStream, response: &Response) -> io::Result<()> {
    let response_str = Protocol::encode_response(response);
    socket.write_all(response_str.as_bytes()).await?;
    Ok(())
} 