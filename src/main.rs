use std::io::{BufRead, BufReader, Write};
// Uncomment this block to pass the first stage
use std::net::{TcpListener, TcpStream};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
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

    let mut first_line = request.first().unwrap().split_whitespace();
    let method = first_line.nth(0).unwrap();
    let path = first_line.nth(0).unwrap();

    println!("{} | {}", method, path);

    if path == "/" {
        _stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).unwrap();
    } else {
        _stream.write("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes()).unwrap();
    }
}
