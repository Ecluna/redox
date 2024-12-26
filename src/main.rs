mod network;
mod protocol;
mod storage;

use network::Server;
use storage::Storage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Redox server starting...");
    
    let storage = Storage::new();
    let server = Server::new(storage);
    
    server.run("127.0.0.1:6379").await?;
    
    Ok(())
}
