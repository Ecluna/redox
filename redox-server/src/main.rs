mod network;
mod storage;

use network::Server;
use storage::Storage;
use std::env;

struct ServerConfig {
    port: u16,
    password: Option<String>,
}

impl ServerConfig {
    fn from_args() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut config = ServerConfig {
            port: 2001,  // 默认端口
            password: None,
        };

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                // 处理端口参数
                port if port.parse::<u16>().is_ok() => {
                    config.port = port.parse().unwrap();
                }
                // 处理密码参数
                arg if arg.starts_with("--password=") => {
                    config.password = Some(arg[11..].to_string());
                }
                _ => {
                    eprintln!("Warning: Unknown argument '{}' ignored", args[i]);
                }
            }
            i += 1;
        }

        config
    }
}

/// 服务器入口函数
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::from_args();

    println!("Redox server starting...");
    if config.password.is_some() {
        println!("Password authentication enabled");
    }
    
    let storage = Storage::new();
    let server = Server::new(storage, config.password);
    
    let mut current_port = config.port;
    loop {
        let addr = format!("127.0.0.1:{}", current_port);
        match server.try_bind(&addr).await {
            Ok(_) => {
                println!("Successfully bound to port {}", current_port);
                server.run(&addr).await?;
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                println!("Port {} is in use, trying {}", current_port, current_port + 1);
                current_port += 1;
            }
            Err(e) => return Err(e.into()),
        }
    }
    
    Ok(())
} 