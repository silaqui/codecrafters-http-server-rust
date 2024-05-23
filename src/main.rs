use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");
                handle_connection(_stream);
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

    println!("Request : \r\n {:?}", request);

    let mut req_iterator = request.iter();

    let mut first_line = req_iterator.nth(0).unwrap().split_whitespace();
    let method = first_line.nth(0).unwrap();
    println!("Method: {} ", method);
    let path = first_line.nth(0).unwrap();
    println!("Path: {} ", path);
    let http_version = first_line.nth(0).unwrap();
    println!("HttpV: {} ", http_version);

    let headers : Vec<_> = req_iterator.take_while(|line| *line != "\r\n").collect();
    println!("Headers: {:?} ", headers);

    if path == "/" {
        _stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).unwrap();
    } else if path.starts_with("/echo") {
        let str = path.split_at(6).1;
        println!("Str: {}", str);

        let response = return_body(str);

        _stream.write(response.as_bytes()).unwrap();
    } else if path.starts_with("/user-agent"){

        let ue = headers.iter().find(|h| h.starts_with("User-Agent:")).unwrap();
        let str = ue.split_whitespace().nth(1).unwrap();

        let response = return_body(str);

        _stream.write(response.as_bytes()).unwrap();
    } else {
        _stream.write("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes()).unwrap();
    }
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
