extern crate serde_json;
extern crate threadpool;

use std::{env, process};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};
use threadpool::ThreadPool;

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
    inspect_config(&conf);

    let tcp_listen_thread_pool = ThreadPool::new(conf.proxy_group.len());

    for proxy in conf.proxy_group {
        tcp_listen_thread_pool.execute(move || {
            listener_bind(proxy);
        });
    }
    // TODO 保持线程存活
    loop {}
}

// 监听绑定
fn listener_bind(proxy: Proxy) {
    let addr = format!(
        "{}:{}",
        proxy.bind,
        proxy.port
    );

    // 创建监听
    let listener = TcpListener::bind(addr)
        .expect("The port is occupied.");

    let worker_thread_pool = ThreadPool::new(proxy.thread_pool_size);

    for stream in listener.incoming() {
        let proxy = proxy.clone();
        worker_thread_pool.execute(move || {
            handle_connection(proxy, stream.unwrap())
        });
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
fn handle_connection(proxy: Proxy, mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    let req = String::from_utf8_lossy(&buffer);
    let arr = req.split(" ");
    let req_vec = arr.collect::<Vec<&str>>();
    let mut filename = req_vec[1];

    let mut symbol = true;
    let mut quote = Option::None;

    for rule in proxy.rules {
        // 确认代理前缀是否匹配
        if filename.starts_with(&rule.0) {
            symbol = false;
            quote = Option::Some(rule.clone());
            if rule.0.eq("/") || rule.0.eq("") {
                continue;
            }
            break;
        }
    }

    if symbol && quote.is_none() {
        return;
    }

    let quote = quote.unwrap();
    let mut access_log = AccessLog::new(
        filename.to_string(),
        quote.1.access_log.to_string()
    );

    // 替换代理前缀
    if quote.0.ne("/") {
        filename = &filename[quote.0.len()..];
    }

    // 替换默认路径
    if filename.eq("") || filename.eq("/") {
        filename = &quote.1.index.as_str();
    }

    access_log.to = filename.to_string();

    if buffer.starts_with("GET".as_bytes()) {
        // 找到对应文件并写入
        let path = format!(
            "{}{}",
            quote.1.path,
            filename
        );
        let content = fs::read_to_string(&path);

        access_log.real_access_path = path;

        let content = match content {
            Ok(c) => {
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                    c.len(),
                    c
                )
            },
            Err(_) => {
                let not_found_page = fs::read_to_string(&quote.1.not_found_page);
                match not_found_page {
                    Ok(n) => {
                        format!(
                            "HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\n\r\n{}",
                            n.len(),
                            n
                        )
                    },
                    Err(_) => {
                        let not_found_page = "<h1>404</h1>";
                        format!(
                            "HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\n\r\n{}",
                            not_found_page.len(),
                            not_found_page
                        )
                    }
                }
            }
        };
        stream.write(content.as_bytes()).unwrap();
    }

    recording_access_log(access_log);
}

// 记录访问日志
fn recording_access_log(access_log: AccessLog) {
    let log_str = format!(
        "echo `date +%Y-%m-%d\\ %H:%M:%S\\ \\ from:{},\\ to:\\ {}`,\\ realAccessPath:\\ {} >> {}",
        access_log.from,
        access_log.to,
        access_log.real_access_path,
        access_log.log_file_path,
    );

    let _ = Command::new("bash")
        .args(&["-c", log_str.as_str()])
        .spawn();
}

// 转换配置
fn parse_config(path: String) -> Config {
    let config_file = File::open(path).unwrap();
    serde_json::from_reader(config_file).unwrap()
}

// 检查配置
fn inspect_config(conf: &Config) {
    for proxy in &conf.proxy_group {
        if proxy.port == 0u32 || proxy.port > 65535u32 {
            prompt_and_exit("Checking Port Settings.");
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    proxy_group: Vec<Proxy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Proxy {
    bind: String,
    port: u32,
    timeout: usize,
    cache: bool,
    thread_pool_size: usize,
    rules: HashMap<String, Rule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Rule {
    path: String,
    index: String,
    access_log: String,
    not_found_page: String,
}

struct AccessLog {
    from: String,
    to: String,
    real_access_path: String,
    log_file_path: String,
}

impl AccessLog {
    fn new(from: String, log_file_path: String) -> AccessLog {
        AccessLog {
            from,
            log_file_path,
            to: String::from(""),
            real_access_path: String::from(""),
        }
    }
}
