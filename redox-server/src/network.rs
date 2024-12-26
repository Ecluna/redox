use crate::storage::Storage;
use redox_protocol::{Command, Protocol, Response};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

pub struct Server {
    storage: Storage,
}

impl Server {
    pub fn new(storage: Storage) -> Self {
        Server { storage }
    }

    pub async fn run(&self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        println!("Server listening on {}", addr);

        loop {
            let (socket, _) = listener.accept().await?;
            let storage = self.storage.clone();
            
            tokio::spawn(async move {
                if let Err(e) = handle_connection(socket, storage).await {
                    eprintln!("Error handling connection: {}", e);
                }
            });
        }
    }
}

async fn handle_connection(
    mut socket: TcpStream,
    storage: Storage,
) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

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
            Command::Set { key, value } => {
                storage.set(key, value).await;
                Response::Ok(None)
            }
            Command::Get { key } => {
                let value = storage.get(&key).await;
                Response::Ok(value)
            }
        };

        let response_str = Protocol::encode_response(&response);
        writer.write_all(response_str.as_bytes()).await?;
    }

    Ok(())
} 