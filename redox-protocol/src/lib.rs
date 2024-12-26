use bytes::{Buf, BufMut, BytesMut};
use std::io;

/// 定义客户端可以发送的命令类型
#[derive(Debug)]
pub enum Command {
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
            // SET 命令格式：SET key value\n
            Command::Set { key, value } => format!("SET {} {}\n", key, value),
            // GET 命令格式：GET key\n
            Command::Get { key } => format!("GET {}\n", key),
        }
    }

    /// 将输入字符串解析为命令
    pub fn decode_command(input: &str) -> Result<Command, String> {
        // 去除输入两端的空白字符
        let input = input.trim();
        // 按空白字符分割命令
        let parts: Vec<&str> = input.split_whitespace().collect();
        
        // 解析命令类型和参数
        match parts.get(0).map(|s| *s) {
            Some(cmd) => match cmd.to_uppercase().as_str() {
                "SET" => {
                    // SET 命令必须包含键和值两个参数
                    if parts.len() != 3 {
                        return Err("SET command requires KEY and VALUE".to_string());
                    }
                    Ok(Command::Set {
                        key: parts[1].to_string(),
                        value: parts[2].to_string(),
                    })
                }
                "GET" => {
                    // GET 命令必须包含键参数
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