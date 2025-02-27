use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::sleep;
use std::time::Duration;

use itertools::Itertools;

use flate2::Compression;
use flate2::write::GzEncoder;
use nom::AsBytes;

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

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = BufReader::new(&mut stream);

    let mut first_line = String::new();
    buffer.read_line(&mut first_line).unwrap();

    let mut first_line = first_line.split_whitespace();
    let _method = first_line.nth(0).unwrap();
    println!("Method: {} ", _method);
    let path = first_line.nth(0).unwrap();
    println!("Path: {} ", path);
    let _http_version = first_line.nth(0).unwrap();
    println!("HttpV: {} ", _http_version);

    let mut headers = HashMap::new();

    loop {
        let mut temp = String::new();
        let r = buffer.read_line(&mut temp).unwrap();
        if r < 3 { //detect empty line
            break;
        }
        let mut iter = temp
            .split(":")
            .map(|s| s.trim());
        let key = iter.nth(0).unwrap().to_owned();
        let val = iter.nth(0).unwrap().to_owned();

        headers.insert(key, val);
    }
    println!("Headers: {:?} ", headers);

    let body = match headers.get("Content-Length") {
        None => None,
        Some(size) => {
            let size = size.parse::<usize>().unwrap(); //Get Content-Length
            println!("Content-Length: {}", size);
            let mut body = vec![0; size];
            buffer.read_exact(&mut body).unwrap();
            println!("Body: {:?} ", body);
            Some(body)
        }
    };

    if body.is_some() {
        println!("Have body: {:?}", body)
    }

    if path == "/" {
        index(&mut stream);
    } else if path.starts_with("/echo") {
        echo(&mut stream, path, headers);
    } else if path.starts_with("/user-agent") {
        user_agent(&mut stream, headers);
    } else if path.starts_with("/delay") {
        delay(&mut stream);
    } else if path.starts_with("/files") {
        if _method == "GET" {
            files(&mut stream, path);
        } else {
            files_save(&mut stream, path, body.unwrap());
        }
    } else {
        not_found(&mut stream);
    }
}

fn index(_stream: &mut TcpStream) {
    _stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).unwrap();
}

fn echo(_stream: &mut TcpStream, path: &str, headers: HashMap<String, String>) {
    let str = path.split_at(6).1;
    println!("Echo: {}", str);

    let encoding = headers
        .get("Accept-Encoding");
    println!("Encoding: {:?}", encoding);

    let gzip_encoding = headers
        .get("Accept-Encoding")
        .map(|str| str.split(",").map(|s|s.trim()))
        .map(|mut split| split.contains(&"gzip"))
        .unwrap_or(false);


    println!("gzip_encoding: {:?}", gzip_encoding);

    let mut response =
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n".to_owned();

    if gzip_encoding {
        response.push_str("Content-Encoding: gzip\r\n");

        let mut encoder = GzEncoder::new(vec![], Compression::default());
        encoder.write_all(str.as_bytes()).unwrap();
        let compressed_buf = encoder.finish().unwrap();

        response.push_str("Content-Length: ");
        let body_length = compressed_buf.len();
        response.push_str(&*body_length.to_string());
        response.push_str("\r\n\r\n");

        let a = response.as_bytes();

        let b = compressed_buf.as_bytes();

        let c = concat_u8(a,b);

        _stream.write(&c).unwrap();

    } else {
        response.push_str("Content-Length: ");
        let body_length = str.len();
        response.push_str(&*body_length.to_string());
        response.push_str("\r\n\r\n");
        response.push_str(str);
        _stream.write(response.as_bytes()).unwrap();
    }

}

pub fn concat_u8(first: &[u8], second: &[u8]) -> Vec<u8> {
    [first, second].concat()
}
fn user_agent(_stream: &mut TcpStream, headers: HashMap<String, String>) {
    let str = headers.get("User-Agent").unwrap();

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

    match File::open(dir + filename) {
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

            _stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).unwrap();
        }
        Err(_) => {
            _stream.write("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes()).unwrap();
        }
    };
}

fn files_save(_stream: &mut TcpStream, path: &str, body: Vec<u8>) {
    let env_args: Vec<String> = env::args().collect();
    println!("Env args : {:?}", env_args);
    let dir = env_args[2].clone();
    println!("dir : {:?}", dir);

    let filename = path.split_at(7).1;
    println!("File: {}", filename);

    let mut file = File::create(dir + filename).unwrap();
    file.write_all(body.as_slice()).unwrap();
    _stream.write("HTTP/1.1 201 Created\r\n\r\n".as_bytes()).unwrap();
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
