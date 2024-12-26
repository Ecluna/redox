use std::io::{self, Write};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:6379").await?;
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    
    println!("Connected to Redox server. Type your commands (e.g., 'SET key value' or 'GET key'):");
    
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