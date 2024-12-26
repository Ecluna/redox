use bytes::{Buf, BufMut, BytesMut};
use std::io;

/// 定义客户端可以发送的命令类型
#[derive(Debug)]
pub enum Command {
    /// AUTH 命令：验证密码
    Auth { password: String },
    /// SET 命令：设置键值对
    Set { key: String, value: String },
    /// GET 命令：获取指定键的值
    Get { key: String },
}

/// 定义服务器的响应类型
#[derive(Debug)]
pub enum Response {
    /// SET 命令成功响应
    Ok,
    /// GET 命令成功响应，包含值
    Value(String),
    /// 错误响应：包含错误信息
    Error(String),
}

/// 协议解析和编码的实现
pub struct Protocol;

impl Protocol {
    /// 将命令编码为字符串格式
    pub fn encode_command(cmd: &Command) -> String {
        match cmd {
            Command::Auth { password } => format!("AUTH {}\n", password),
            Command::Set { key, value } => format!("SET {} {}\n", key, value),
            Command::Get { key } => format!("GET {}\n", key),
        }
    }

    /// 将输入字符串解析为命令
    pub fn decode_command(input: &str) -> Result<Command, String> {
        let input = input.trim();
        let parts: Vec<&str> = input.split_whitespace().collect();
        
        match parts.get(0).map(|s| *s) {
            Some(cmd) => match cmd.to_uppercase().as_str() {
                "AUTH" => {
                    if parts.len() != 2 {
                        return Err("AUTH command requires password".to_string());
                    }
                    Ok(Command::Auth {
                        password: parts[1].to_string(),
                    })
                }
                "SET" => {
                    if parts.len() != 3 {
                        return Err("SET command requires KEY and VALUE".to_string());
                    }
                    Ok(Command::Set {
                        key: parts[1].to_string(),
                        value: parts[2].to_string(),
                    })
                }
                "GET" => {
                    if parts.len() != 2 {
                        return Err("GET command requires KEY".to_string());
                    }
                    Ok(Command::Get {
                        key: parts[1].to_string(),
                    })
                }
                _ => Err("Unknown command".to_string()),
            },
            None => Err("Empty command".to_string()),
        }
    }

    /// 将响应编码为字符串格式
    pub fn encode_response(resp: &Response) -> String {
        match resp {
            // SET 命令成功返回 OK
            Response::Ok => "OK\n".to_string(),
            // GET 命令成功，有值时直接返回值
            Response::Value(value) => format!("{}\n", value),
            // 错误响应
            Response::Error(err) => format!("ERR {}\n", err),
        }
    }
} 