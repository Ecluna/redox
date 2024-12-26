use bytes::{Buf, BufMut, BytesMut};
use std::io;

#[derive(Debug)]
pub enum Command {
    Set { key: String, value: String },
    Get { key: String },
}

#[derive(Debug)]
pub enum Response {
    Ok(Option<String>),
    Error(String),
}

pub struct Protocol;

impl Protocol {
    pub fn encode_command(cmd: &Command) -> String {
        match cmd {
            Command::Set { key, value } => format!("SET {} {}\n", key, value),
            Command::Get { key } => format!("GET {}\n", key),
        }
    }

    pub fn decode_command(input: &str) -> Result<Command, String> {
        let input = input.trim();
        let parts: Vec<&str> = input.split_whitespace().collect();
        
        match parts.get(0).map(|s| *s) {
            Some(cmd) => match cmd.to_uppercase().as_str() {
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

    pub fn encode_response(resp: &Response) -> String {
        match resp {
            Response::Ok(Some(value)) => format!("OK {}\n", value),
            Response::Ok(None) => "OK\n".to_string(),
            Response::Error(err) => format!("ERR {}\n", err),
        }
    }
} 