use std::env;
use std::io::{self, Write};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

/// 客户端入口函数
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从命令行参数获取端口，如果没有则使用默认端口
    let port = env::args()
        .nth(1)
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(2001);

    let addr = format!("127.0.0.1:{}", port);
    let mut stream = match TcpStream::connect(&addr).await {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Failed to connect to {}: {}", addr, e);
            return Err(e.into());
        }
    };

    println!("Connected to Redox server at {}. Type your commands (e.g., 'SET key value' or 'GET key'):", addr);
    
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    
    loop {
        print!("> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if input.trim().eq_ignore_ascii_case("quit") {
            break;
        }
        
        writer.write_all(input.as_bytes()).await?;
        
        let mut response = String::new();
        reader.read_line(&mut response).await?;
        print!("< {}", response);
    }
    
    Ok(())
} 