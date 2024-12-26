mod storage;

use storage::Storage;

#[tokio::main]
async fn main() {
    println!("Redox server starting...");
    let storage = Storage::new();
    
    // TODO: 实现网络服务器
}
