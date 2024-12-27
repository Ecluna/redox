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
        }
    }
} 