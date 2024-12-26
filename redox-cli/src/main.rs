use std::io::{self, Write};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

/// 客户端入口函数
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 连接到服务器
    let mut stream = TcpStream::connect("127.0.0.1:6379").await?;
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    
    println!("Connected to Redox server. Type your commands (e.g., 'SET key value' or 'GET key'):");
    
    // 命令处理循环
    loop {
        // 显示提示符
        print!("> ");
        io::stdout().flush()?;
        
        // 读取用户输入
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        // 检查是否退出
        if input.trim().eq_ignore_ascii_case("quit") {
            break;
        }
        
        // 发送命令到服务器
        writer.write_all(input.as_bytes()).await?;
        
        // 读取服务器响应
        let mut response = String::new();
        reader.read_line(&mut response).await?;
        print!("< {}", response);
    }
    
    Ok(())
} 