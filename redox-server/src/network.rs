use crate::storage::Storage;
use redox_protocol::{Command, Protocol, Response, RedoxValue};
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
        authenticated: password.is_none(),
        requires_auth: password.is_some(),
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
                handle_auth(&mut state, &password, &input_password)
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
        };

        let response_str = Protocol::encode_response(&response);
        writer.write_all(response_str.as_bytes()).await?;
    }

    Ok(())
}

fn handle_auth(state: &mut ConnectionState, server_password: &Option<Arc<String>>, input_password: &str) -> Response {
    if !state.requires_auth {
        Response::Error("Authentication not required".to_string())
    } else if let Some(correct_password) = server_password {
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