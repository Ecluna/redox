use redox_protocol::{Command, Protocol, Response};
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;
use std::thread;

fn connect_with_retry(addr: &str, max_retries: u32) -> io::Result<TcpStream> {
    for i in 0..max_retries {
        match TcpStream::connect(addr) {
            Ok(stream) => {
                println!("Successfully connected to server");
                return Ok(stream);
            }
            Err(e) => {
                if i < max_retries - 1 {
                    println!("Connection attempt {} failed: {}. Retrying...", i + 1, e);
                    thread::sleep(Duration::from_secs(1));
                } else {
                    println!("Failed to connect after {} attempts: {}", max_retries, e);
                    return Err(e);
                }
            }
        }
    }
    unreachable!()
}

struct Test {
    command: &'static str,
    expected: Vec<&'static str>,
}

impl Test {
    fn new(command: &'static str, expected: &'static str) -> Self {
        Test {
            command,
            expected: vec![expected],
        }
    }

    fn with_multiple(command: &'static str, expected: Vec<&'static str>) -> Self {
        Test {
            command,
            expected,
        }
    }

    fn matches(&self, response: &str) -> bool {
        self.expected.iter().any(|exp| exp == &response)
    }
}

fn main() {
    println!("Attempting to connect to server...");
    let mut stream = connect_with_retry("127.0.0.1:3000", 5)
        .expect("Failed to connect to server after multiple attempts");
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    let tests = vec![
        // 认证测试
        Test::new("auth mypassword", "OK"),
        
        // 字符串操作测试
        Test::new("set testkey hello", "OK"),
        Test::new("get testkey", "hello"),
        Test::new("set counter 1", "OK"),
        Test::new("get counter", "1"),
        
        // 列表操作测试
        Test::new("lpush mylist world", "1"),
        Test::new("lpush mylist hello", "2"),
        Test::new("lrange mylist 0 -1", "hello world"),
        Test::new("rpush mylist !", "3"),
        Test::new("lpop mylist", "hello"),
        Test::new("rpop mylist", "!"),
        
        // 集合操作测试
        Test::new("sadd myset member1", "1"),
        Test::new("sadd myset member2", "1"),
        Test::new("sadd myset member1", "0"),
        Test::with_multiple("smembers myset", vec!["member1 member2", "member2 member1"]),
        Test::new("sismember myset member1", "1"),
        Test::new("srem myset member1", "1"),
        
        // 哈希表操作测试
        Test::new("hset user:1 name luna", "1"),
        Test::new("hset user:1 age 21", "1"),
        Test::new("hget user:1 name", "luna"),
        Test::with_multiple("hgetall user:1", vec!["name luna age 21", "age 21 name luna"]),
        Test::new("hdel user:1 age", "1"),
        
        // 有序集合操作测试
        Test::new("zadd scores 66 user1", "1"),
        Test::new("zadd scores 77 user2", "1"),
        Test::new("zadd scores 88 user3", "1"),
        Test::new("zadd scores 99 user5", "1"),
        Test::new("zadd scores 100 user4", "1"),
        Test::new("zrange scores 0 -1", "user1 66 user2 77 user3 88 user4 100 user5 99"),
        Test::new("zrangebyscore scores 80 100", "user3 88 user4 100 user5 99"),
        
        // 批量操作测试
        Test::new("mset k1 v1 k2 v2", "2"),
        Test::new("mget k1 k2", "v1 v2"),
        
        // 信息查询测试
        Test::new("info", "hashes: 1 keys: 8 lists: 1 sets: 1 strings: 4 zsets: 1"),
    ];

    let mut success = 0;
    let total = tests.len();

    for (i, test) in tests.iter().enumerate() {
        println!("\n=== Test {} ===", i + 1);
        println!("Command: {}", test.command);
        
        // 发送命令
        let cmd = format!("{}\n", test.command);
        stream.write_all(cmd.as_bytes()).expect("Failed to send command");
        
        // 读取响应
        let mut response = String::new();
        reader.read_line(&mut response).expect("Failed to read response");
        let response = response.trim();
        
        println!("Expected one of: {:?}", test.expected);
        println!("Got: {}", response);
        
        if test.matches(response) {
            println!("✅ Test passed");
            success += 1;
        } else {
            println!("❌ Test failed");
        }

        thread::sleep(Duration::from_millis(100));
    }

    println!("\n=== Test Summary ===");
    println!("Total tests: {}", total);
    println!("Passed: {}", success);
    println!("Failed: {}", total - success);
    println!("Success rate: {:.1}%", (success as f64 / total as f64) * 100.0);
} 