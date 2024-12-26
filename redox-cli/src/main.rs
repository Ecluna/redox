use std::env;
use std::io::{self, Write};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::sleep;

const MAX_RETRIES: u32 = 3;
const RETRY_DELAY: Duration = Duration::from_secs(1);

/// 客户端入口函数
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = env::args()
        .nth(1)
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(2001);

    let addr = format!("127.0.0.1:{}", port);
    let stream = connect_with_retry(&addr).await?;
    let (reader, writer) = stream.into_split();
    
    println!("Connected to Redox server at {}. Type your commands (e.g., 'SET key value' or 'GET key'):", addr);
    println!("Type 'quit' to exit.");
    
    let mut reader = BufReader::new(reader);
    let mut writer = writer;
    
    loop {
        print!("> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let trimmed = input.trim();
        if trimmed.eq_ignore_ascii_case("quit") {
            println!("Goodbye!");
            break;
        }
        
        if trimmed.is_empty() {
            continue;
        }

        if let Err(e) = writer.write_all(input.as_bytes()).await {
            eprintln!("Error sending command: {}", e);
            break;
        }
        writer.flush().await?;
        
        let mut response = String::new();
        if let Err(e) = reader.read_line(&mut response).await {
            eprintln!("Error reading response: {}", e);
            break;
        }
        print!("< {}", response);
    }
    
    Ok(())
}

/// 尝试连接服务器，带重试机制
async fn connect_with_retry(addr: &str) -> Result<TcpStream, Box<dyn std::error::Error>> {
    let mut retries = 0;
    loop {
        match TcpStream::connect(addr).await {
            Ok(stream) => return Ok(stream),
            Err(e) => {
                retries += 1;
                if retries >= MAX_RETRIES {
                    return Err(format!("Failed to connect after {} attempts: {}", MAX_RETRIES, e).into());
                }
                eprintln!("Connection attempt {} failed: {}. Retrying in {} seconds...", 
                    retries, e, RETRY_DELAY.as_secs());
                sleep(RETRY_DELAY).await;
            }
        }
    }
} 