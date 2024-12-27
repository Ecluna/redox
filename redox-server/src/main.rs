mod network;
mod storage;

use network::Server;
use storage::Storage;
use clap::Parser;
use std::time::Duration;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct ServerConfig {
    /// Port to listen on (default: 2001)
    #[arg(short = 'P', long, default_value_t = 2001)]
    port: u16,

    /// Password for authentication
    #[arg(short = 'p', long)]
    password: Option<String>,

    /// Data file path for persistence
    #[arg(short = 'f', long)]
    data_file: Option<String>,

    /// Auto-save interval in seconds
    #[arg(short = 'i', long, default_value_t = 60)]
    save_interval: u64,
}

/// 服务器入口函数
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::parse();

    let persistence = config.data_file.map(|path| {
        Persistence::new(
            path,
            Duration::from_secs(config.save_interval),
        )
    });

    let storage = Storage::new(persistence);
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