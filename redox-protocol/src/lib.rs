/// 支持的数据类型
#[derive(Debug, Clone)]
pub enum RedoxValue {
    String(String),
    List(Vec<String>),
    Set(std::collections::HashSet<String>),
    Hash(std::collections::HashMap<String, String>),
    SortedSet(std::collections::BTreeMap<String, f64>), // 键为成员，值为分数
}

/// 命令类型
#[derive(Debug)]
pub enum Command {
    // 认证
    Auth { password: String },
    
    // 字符串操作
    Set { key: String, value: String },
    Get { key: String },
    
    // 列表操作
    LPush { key: String, value: String },
    RPush { key: String, value: String },
    LPop { key: String },
    RPop { key: String },
    LRange { key: String, start: i64, stop: i64 },
    
    // 集合操作
    SAdd { key: String, member: String },
    SRem { key: String, member: String },
    SMembers { key: String },
    SIsMember { key: String, member: String },
    
    // 哈希操作
    HSet { key: String, field: String, value: String },
    HGet { key: String, field: String },
    HGetAll { key: String },
    HDel { key: String, field: String },
    
    // 有序集合操作
    ZAdd { key: String, score: f64, member: String },
    ZRem { key: String, member: String },
    ZRange { key: String, start: i64, stop: i64 },
    ZRangeByScore { key: String, min: f64, max: f64 },
}

/// 响应类型
#[derive(Debug)]
pub enum Response {
    Ok,
    Value(RedoxValue),
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
                "LPUSH" => {
                    if parts.len() != 3 {
                        return Err("LPUSH command requires KEY and VALUE".to_string());
                    }
                    Ok(Command::LPush {
                        key: parts[1].to_string(),
                        value: parts[2].to_string(),
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
            Response::Ok => "OK\n".to_string(),
            Response::Value(value) => match value {
                RedoxValue::String(s) => format!("{}\n", s),
                RedoxValue::List(list) => format!("{}\n", list.join(" ")),
                RedoxValue::Set(set) => format!("{}\n", set.iter().collect::<Vec<_>>().join(" ")),
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
        }
    }
} 