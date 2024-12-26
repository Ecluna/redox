mod network;
mod storage;

use network::Server;
use storage::Storage;
use std::env;

/// 服务器入口函数
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从命令行参数获取端口，如果没有则使用默认端口
    let port = env::args()
        .nth(1)
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(2001);

    println!("Redox server starting...");
    
    let storage = Storage::new();
    let server = Server::new(storage);
    
    // 尝试绑定端口，如果被占用则尝试下一个端口
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