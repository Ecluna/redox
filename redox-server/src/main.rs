mod network;
mod storage;

use network::Server;
use storage::Storage;
use std::env;

/// 服务器入口函数
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    // 解析命令行参数
    let (port, password) = match args.len() {
        1 => (2001, None), // 无参数：默认端口，无密码
        2 => {
            // 一个参数：可能是端口或密码
            if args[1].starts_with("--password=") {
                (2001, Some(args[1][11..].to_string()))
            } else {
                (args[1].parse::<u16>().unwrap_or(2001), None)
            }
        }
        3 => {
            // 两个参数：端口和密码
            if args[2].starts_with("--password=") {
                (
                    args[1].parse::<u16>().unwrap_or(2001),
                    Some(args[2][11..].to_string()),
                )
            } else {
                (2001, None)
            }
        }
        _ => (2001, None),
    };

    println!("Redox server starting...");
    if password.is_some() {
        println!("Password authentication enabled");
    }
    
    let storage = Storage::new();
    let server = Server::new(storage, password);
    
    let mut current_port = port;
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