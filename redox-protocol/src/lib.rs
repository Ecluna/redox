use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// 支持的数据类型
/// 使用 serde 进行序列化和反序列化，支持 JSON 格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RedoxValue {
    /// 字符串类型
    String(String),
    /// 列表类型，使用 Vec 实现
    List(Vec<String>),
    /// 集合类型，使用 HashSet 实现，保证元素唯一性
    Set(std::collections::HashSet<String>),
    /// 哈希表类型，键值对存储
    Hash(std::collections::HashMap<String, String>),
    /// 有序集合类型，使用 BTreeMap 实现
    /// 键为成员，值为分数，通过分数自动排序
    SortedSet(std::collections::BTreeMap<String, f64>),
}

/// 命令类型
/// 定义所有支持的命令及其参数
#[derive(Debug)]
pub enum Command {
    /// 认证命令
    Auth { password: String },
    
    // 字符串操作
    /// SET key value
    Set { key: String, value: String },
    /// GET key
    Get { key: String },
    
    // 列表操作
    /// LPUSH key value
    LPush { key: String, value: String },
    /// RPUSH key value
    RPush { key: String, value: String },
    /// LPOP key
    LPop { key: String },
    /// RPOP key
    RPop { key: String },
    /// LRANGE key start stop
    LRange { key: String, start: i64, stop: i64 },
    
    // 集合操作
    /// SADD key member
    SAdd { key: String, member: String },
    /// SREM key member
    SRem { key: String, member: String },
    /// SMEMBERS key
    SMembers { key: String },
    /// SISMEMBER key member
    SIsMember { key: String, member: String },
    
    // 哈希操作
    /// HSET key field value
    HSet { key: String, field: String, value: String },
    /// HGET key field
    HGet { key: String, field: String },
    /// HGETALL key
    HGetAll { key: String },
    /// HDEL key field
    HDel { key: String, field: String },
    
    // 有序集合操作
    /// ZADD key score member
    ZAdd { key: String, score: f64, member: String },
    /// ZREM key member
    ZRem { key: String, member: String },
    /// ZRANGE key start stop
    ZRange { key: String, start: i64, stop: i64 },
    /// ZRANGEBYSCORE key min max
    ZRangeByScore { key: String, min: f64, max: f64 },
    MSet(Vec<(String, String)>),  // 批量设置
    MGet(Vec<String>),           // 批量获取
    Info,                        // 获取信息
}

/// 响应类型
#[derive(Debug)]
pub enum Response {
    /// 操作成功，无返回值
    Ok,
    /// 操作成功，返回值
    Value(RedoxValue),
    /// 操作失败，错误信息
    Error(String),
    Array(Vec<Option<String>>),  // 用于 MGET 的响应
    Integer(usize),              // 用于 MSET 的响应
    Info(HashMap<String, String>), // 用于 INFO 的响应
}

/// 协议解析和编码的实现
pub struct Protocol;

impl Protocol {
    /// 将命令编码为字符串格式
    /// 
    /// # Arguments
    /// * `cmd` - 要编码的命令
    /// 
    /// # Returns
    /// 编码后的字符串，以换行符结尾
    pub fn encode_command(cmd: &Command) -> String {
        match cmd {
            Command::Auth { password } => format!("AUTH {}\n", password),
            Command::Set { key, value } => format!("SET {} {}\n", key, value),
            Command::Get { key } => format!("GET {}\n", key),
            Command::LPush { key, value } => format!("LPUSH {} {}\n", key, value),
            Command::RPush { key, value } => format!("RPUSH {} {}\n", key, value),
            Command::LPop { key } => format!("LPOP {}\n", key),
            Command::RPop { key } => format!("RPOP {}\n", key),
            Command::LRange { key, start, stop } => format!("LRANGE {} {} {}\n", key, start, stop),
            Command::SAdd { key, member } => format!("SADD {} {}\n", key, member),
            Command::SRem { key, member } => format!("SREM {} {}\n", key, member),
            Command::SMembers { key } => format!("SMEMBERS {}\n", key),
            Command::SIsMember { key, member } => format!("SISMEMBER {} {}\n", key, member),
            Command::HSet { key, field, value } => format!("HSET {} {} {}\n", key, field, value),
            Command::HGet { key, field } => format!("HGET {} {}\n", key, field),
            Command::HGetAll { key } => format!("HGETALL {}\n", key),
            Command::HDel { key, field } => format!("HDEL {} {}\n", key, field),
            Command::ZAdd { key, score, member } => format!("ZADD {} {} {}\n", key, score, member),
            Command::ZRem { key, member } => format!("ZREM {} {}\n", key, member),
            Command::ZRange { key, start, stop } => format!("ZRANGE {} {} {}\n", key, start, stop),
            Command::ZRangeByScore { key, min, max } => format!("ZRANGEBYSCORE {} {} {}\n", key, min, max),
            Command::MSet(pairs) => {
                let mut cmd = String::new();
                for (key, value) in pairs {
                    cmd.push_str(&format!("SET {} {}\n", key, value));
                }
                cmd
            },
            Command::MGet(keys) => {
                let mut cmd = String::new();
                for key in keys {
                    cmd.push_str(&format!("GET {}\n", key));
                }
                cmd
            },
            Command::Info => "INFO\n".to_string(),
        }
    }

    /// 将输入字符串解析为命令
    /// 
    /// # Arguments
    /// * `input` - 输入的命令字符串
    /// 
    /// # Returns
    /// * `Ok(Command)` - 解析成功的命令
    /// * `Err(String)` - 解析错误信息
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
                "LPUSH" => {
                    if parts.len() != 3 {
                        return Err("LPUSH command requires KEY and VALUE".to_string());
                    }
                    Ok(Command::LPush {
                        key: parts[1].to_string(),
                        value: parts[2].to_string(),
                    })
                }
                "RPUSH" => {
                    if parts.len() != 3 {
                        return Err("RPUSH command requires KEY and VALUE".to_string());
                    }
                    Ok(Command::RPush {
                        key: parts[1].to_string(),
                        value: parts[2].to_string(),
                    })
                }
                "LPOP" => {
                    if parts.len() != 2 {
                        return Err("LPOP command requires KEY".to_string());
                    }
                    Ok(Command::LPop {
                        key: parts[1].to_string(),
                    })
                }
                "RPOP" => {
                    if parts.len() != 2 {
                        return Err("RPOP command requires KEY".to_string());
                    }
                    Ok(Command::RPop {
                        key: parts[1].to_string(),
                    })
                }
                "LRANGE" => {
                    if parts.len() != 4 {
                        return Err("LRANGE command requires KEY, START and STOP".to_string());
                    }
                    let start = parts[2].parse::<i64>()
                        .map_err(|_| "Invalid START index".to_string())?;
                    let stop = parts[3].parse::<i64>()
                        .map_err(|_| "Invalid STOP index".to_string())?;
                    Ok(Command::LRange {
                        key: parts[1].to_string(),
                        start,
                        stop,
                    })
                }
                "SADD" => {
                    if parts.len() != 3 {
                        return Err("SADD command requires KEY and MEMBER".to_string());
                    }
                    Ok(Command::SAdd {
                        key: parts[1].to_string(),
                        member: parts[2].to_string(),
                    })
                }
                "SREM" => {
                    if parts.len() != 3 {
                        return Err("SREM command requires KEY and MEMBER".to_string());
                    }
                    Ok(Command::SRem {
                        key: parts[1].to_string(),
                        member: parts[2].to_string(),
                    })
                }
                "SMEMBERS" => {
                    if parts.len() != 2 {
                        return Err("SMEMBERS command requires KEY".to_string());
                    }
                    Ok(Command::SMembers {
                        key: parts[1].to_string(),
                    })
                }
                "SISMEMBER" => {
                    if parts.len() != 3 {
                        return Err("SISMEMBER command requires KEY and MEMBER".to_string());
                    }
                    Ok(Command::SIsMember {
                        key: parts[1].to_string(),
                        member: parts[2].to_string(),
                    })
                }
                "HSET" => {
                    if parts.len() != 4 {
                        return Err("HSET command requires KEY, FIELD and VALUE".to_string());
                    }
                    Ok(Command::HSet {
                        key: parts[1].to_string(),
                        field: parts[2].to_string(),
                        value: parts[3].to_string(),
                    })
                }
                "HGET" => {
                    if parts.len() != 3 {
                        return Err("HGET command requires KEY and FIELD".to_string());
                    }
                    Ok(Command::HGet {
                        key: parts[1].to_string(),
                        field: parts[2].to_string(),
                    })
                }
                "HDEL" => {
                    if parts.len() != 3 {
                        return Err("HDEL command requires KEY and FIELD".to_string());
                    }
                    Ok(Command::HDel {
                        key: parts[1].to_string(),
                        field: parts[2].to_string(),
                    })
                }
                "HGETALL" => {
                    if parts.len() != 2 {
                        return Err("HGETALL command requires KEY".to_string());
                    }
                    Ok(Command::HGetAll {
                        key: parts[1].to_string(),
                    })
                }
                "ZADD" => {
                    if parts.len() != 4 {
                        return Err("ZADD command requires KEY, SCORE and MEMBER".to_string());
                    }
                    let score = parts[2].parse::<f64>()
                        .map_err(|_| "Invalid SCORE".to_string())?;
                    Ok(Command::ZAdd {
                        key: parts[1].to_string(),
                        score,
                        member: parts[3].to_string(),
                    })
                }
                "ZREM" => {
                    if parts.len() != 3 {
                        return Err("ZREM command requires KEY and MEMBER".to_string());
                    }
                    Ok(Command::ZRem {
                        key: parts[1].to_string(),
                        member: parts[2].to_string(),
                    })
                }
                "ZRANGE" => {
                    if parts.len() != 4 {
                        return Err("ZRANGE command requires KEY, START and STOP".to_string());
                    }
                    let start = parts[2].parse::<i64>()
                        .map_err(|_| "Invalid START index".to_string())?;
                    let stop = parts[3].parse::<i64>()
                        .map_err(|_| "Invalid STOP index".to_string())?;
                    Ok(Command::ZRange {
                        key: parts[1].to_string(),
                        start,
                        stop,
                    })
                }
                "ZRANGEBYSCORE" => {
                    if parts.len() != 4 {
                        return Err("ZRANGEBYSCORE command requires KEY, MIN and MAX".to_string());
                    }
                    let min = parts[2].parse::<f64>()
                        .map_err(|_| "Invalid MIN score".to_string())?;
                    let max = parts[3].parse::<f64>()
                        .map_err(|_| "Invalid MAX score".to_string())?;
                    Ok(Command::ZRangeByScore {
                        key: parts[1].to_string(),
                        min,
                        max,
                    })
                }
                "MSET" => {
                    if parts.len() < 3 || parts.len() % 2 != 1 {
                        return Err("MSET requires key value pairs".to_string());
                    }
                    let mut pairs = Vec::new();
                    for chunk in parts[1..].chunks(2) {
                        pairs.push((chunk[0].to_string(), chunk[1].to_string()));
                    }
                    Ok(Command::MSet(pairs))
                }
                "MGET" => {
                    if parts.len() < 2 {
                        return Err("MGET requires at least one key".to_string());
                    }
                    Ok(Command::MGet(parts[1..].iter().map(|s| s.to_string()).collect()))
                }
                "INFO" => Ok(Command::Info),
                _ => Err(format!("Unknown command: {}", parts[0])),
            },
            None => Err("Empty command".to_string()),
        }
    }

    /// 将响应编码为字符串格式
    /// 
    /// # Arguments
    /// * `resp` - 要编码的响应
    /// 
    /// # Returns
    /// 编码后的字符串，以换行符结尾
    pub fn encode_response(resp: &Response) -> String {
        match resp {
            Response::Ok => "OK\n".to_string(),
            Response::Value(value) => match value {
                RedoxValue::String(s) => format!("{}\n", s),
                RedoxValue::List(list) => format!("{}\n", list.join(" ")),
                RedoxValue::Set(set) => format!("{}\n", set.iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")),
                RedoxValue::Hash(hash) => {
                    let pairs: Vec<String> = hash
                        .iter()
                        .map(|(k, v)| format!("{} {}", k, v))
                        .collect();
                    format!("{}\n", pairs.join(" "))
                },
                RedoxValue::SortedSet(zset) => {
                    let members: Vec<String> = zset
                        .iter()
                        .map(|(member, score)| format!("{} {}", member, score))
                        .collect();
                    format!("{}\n", members.join(" "))
                },
            },
            Response::Error(err) => format!("ERR {}\n", err),
            Response::Array(items) => {
                let items: Vec<String> = items.iter()
                    .map(|item| item.as_ref().map(|s| s.to_string()).unwrap_or("NIL".to_string()))
                    .collect();
                format!("{}\n", items.join(" "))
            },
            Response::Integer(value) => format!("{}\n", value),
            Response::Info(info) => {
                let mut result = Vec::new();
                for (key, value) in info {
                    result.push(format!("{}: {}", key, value));
                }
                result.sort();  // 保证顺序一致
                result.join("\n") + "\n"  // 所有信息一次性返回
            },
        }
    }
} 