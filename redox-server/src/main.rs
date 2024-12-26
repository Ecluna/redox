mod network;
mod storage;

use network::Server;
use storage::Storage;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct ServerConfig {
    /// Port to listen on (default: 2001)
    #[arg(short, long, default_value_t = 2001)]
    port: u16,

    /// Password for authentication
    #[arg(short, long)]
    password: Option<String>,
}

/// 服务器入口函数
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::parse();

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