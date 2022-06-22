use std::env;
use std::fs;
use std::path::Path;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn main() {
    let path:Vec<String> = env::args().collect();
    let path = &path[1].as_str();
    // 验证路径是否存在
    if !Path::new(&path).exists() {
        panic!("Path does not exist.");
    }
    // 监听对应端口
    let listener = TcpListener::bind("127.0.0.1:12004").unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(path.to_string(), stream);
    }
}

/*处理请求*/
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
            },
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
