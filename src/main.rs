use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::sleep;
use std::time::Duration;

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");
                std::thread::spawn(|| handle_connection(_stream));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut _stream: TcpStream) {
    let buffer = BufReader::new(&mut _stream);
    let request: Vec<_> = buffer.lines().map(|result| result
        .unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    // println!("Request : \r\n {:?}", request);

    let mut req_iterator = request.iter();

    let mut first_line = req_iterator.nth(0).unwrap().split_whitespace();
    let _method = first_line.nth(0).unwrap();
    // println!("Method: {} ", method);
    let path = first_line.nth(0).unwrap();
    // println!("Path: {} ", path);
    let _http_version = first_line.nth(0).unwrap();
    // println!("HttpV: {} ", http_version);
    let headers: Vec<_> = req_iterator.take_while(|line| *line != "\r\n").collect();
    // println!("Headers: {:?} ", headers);

    if path == "/" {
        index(&mut _stream);
    } else if path.starts_with("/echo") {
        echo(&mut _stream, path);
    } else if path.starts_with("/user-agent") {
        user_agent(&mut _stream, headers);
    } else if path.starts_with("/delay") {
        delay(&mut _stream);
    } else if path.starts_with("/files") {
        files(&mut _stream, path);
    } else {
        not_found(&mut _stream);
    }
}

fn index(_stream: &mut TcpStream) {
    _stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).unwrap();
}

fn echo(_stream: &mut TcpStream, path: &str) {
    let str = path.split_at(6).1;
    println!("Echo: {}", str);

    let response = return_body(str);
    _stream.write(response.as_bytes()).unwrap();
}

fn user_agent(_stream: &mut TcpStream, headers: Vec<&String>) {
    let ue = headers.iter().find(|h| h.starts_with("User-Agent:")).unwrap();
    let str = ue.split_whitespace().nth(1).unwrap();

    let response = return_body(str);
    _stream.write(response.as_bytes()).unwrap();
}

fn delay(_stream: &mut TcpStream) {
    println!("Waiting Starts");
    sleep(Duration::from_secs(5));
    println!("Waiting Ends");

    _stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).unwrap();
}

fn files(_stream: &mut TcpStream, path: &str) {

    let env_args: Vec<String> = env::args().collect();
    println!("Env args : {:?}", env_args);
    let dir = env_args[2].clone();
    println!("dir : {:?}", dir);

    let filename = path.split_at(7).1;
    println!("File: {}", filename);

    match File::open(dir +  filename) {
        Ok(mut file) => {
            let buffer = &mut String::new();

            match file.read_to_string(buffer) {
                Ok(size) => {
                    let mut response =
                        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: ".to_owned();
                    response.push_str(&*size.to_string());
                    response.push_str("\r\n\r\n");
                    response.push_str(buffer);
                    _stream.write(response.as_bytes()).unwrap();
                }
                Err(_) => {
                    _stream.write("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes()).unwrap();
                }
            }

            _stream.write("HTTP/1.1 200 Not Found\r\n\r\n".as_bytes()).unwrap();
        }
        Err(_) => {
            _stream.write("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes()).unwrap();
        }
    };
}

fn not_found(_stream: &mut TcpStream) {
    _stream.write("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes()).unwrap();
}


fn return_body(str: &str) -> String {
    let body_length = str.len();
    let mut response =
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: ".to_owned();
    response.push_str(&*body_length.to_string());
    response.push_str("\r\n\r\n");
    response.push_str(str);
    response
}
