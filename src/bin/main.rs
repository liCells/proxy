extern crate serde_json;

use std::{env, process};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

use proxy::ThreadPool;

fn main() {
    let path: Vec<String> = env::args().collect();
    let path = &path[1].as_str();
    let file = Path::new(&path);
    // 验证路径是否存在且是否是个文件
    if !file.exists() || !file.is_file() {
        panic!("config file does not exist.");
    }

    // 处理并检查配置文件
    let conf = parse_config(path.to_string());
    inspect_config(conf);

    let pool = ThreadPool::new(4);

    // 监听对应端口
    let listener = TcpListener::bind("0.0.0.0:12004").expect("The port is occupied.");

    for stream in listener.incoming() {
        pool.execute(move||{
            let stream = stream.unwrap();
            handle_connection(String::from("/Users/lz/code/projects/github/proxy/html/"), stream)
        })
    }
}

// 转换配置
fn parse_config(path: String) -> Config {
    let config_file = File::open(path).unwrap();
    serde_json::from_reader(config_file).unwrap()
}

// 检查配置
fn inspect_config(conf: Config) {
    for proxy in conf.proxy_group {
        if proxy.port == 0u32 || proxy.port > 65535u32 {
            prompt_and_exit("Checking Port Settings.");
        }
    }
}

// 提示并退出
fn prompt_and_exit(msg: &str) {
    let _ = Command::new("bash")
        .args(&["-c", format!("echo {}", msg).as_str()])
        .spawn();
    process::exit(1);
}

// 处理请求
fn handle_connection(path: String, mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    let req = String::from_utf8_lossy(&buffer);
    let arr = req.split(" ");
    let filename = arr.collect::<Vec<&str>>()[1];

    if buffer.starts_with("GET".as_bytes()) {
        // 找到对应文件并写入
        println!("{}", &path);
        println!("{}", filename);
        let content = fs::read_to_string(path + filename);
        let content = match content {
            Ok(c) => {
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                    c.len(),
                    c
                )
            }
            Err(_c) => {
                let not_found_page = "<h1>404</h1>";
                format!(
                    "HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\n\r\n{}",
                    not_found_page.len(),
                    not_found_page
                )
            }
        };
        stream.write(content.as_bytes()).unwrap();
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    thread_pool_size: usize,
    proxy_group: Vec<Proxy>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Proxy {
    port: u32,
    context: String,
    path: String,
    index: String,
    timeout: usize,
    cache: bool,
}
