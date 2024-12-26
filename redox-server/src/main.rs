mod network;
mod storage;

use network::Server;
use storage::Storage;

/// 服务器入口函数
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Redox server starting...");
    
    // 创建存储实例
    let storage = Storage::new();
    // 创建服务器实例
    let server = Server::new(storage);
    
    // 启动服务器，监听指定地址
    server.run("127.0.0.1:6379").await?;
    
    Ok(())
} 