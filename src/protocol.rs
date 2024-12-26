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
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        match parts.as_slice() {
            ["SET", key, value] => Ok(Command::Set {
                key: key.to_string(),
                value: value.to_string(),
            }),
            ["GET", key] => Ok(Command::Get {
                key: key.to_string(),
            }),
            _ => Err("Invalid command format".to_string()),
        }
    }

    pub fn encode_response(resp: &Response) -> String {
        match resp {
            Response::Ok(Some(value)) => format!("OK {}\n", value),
            Response::Ok(None) => "NIL\n".to_string(),
            Response::Error(err) => format!("ERR {}\n", err),
        }
    }
} 