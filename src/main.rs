extern crate clap;

mod threads;

use clap::{Arg, App};

use std::io::{Read, Write, BufReader, BufRead};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;

use crate::threads::ThreadPool;

fn main() {

    let matches = App::new("Simple Http Server")
        .version("0.1.0")
        .author("Kyle R. <kyle@banyango.com>")
        .about("Sets up a simple http server for serving static content from a directory")
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .required(false)
            .help("Set the port to serve on")).get_matches();    

    let port = matches.value_of("port").unwrap_or("3000");
    
    println!("Started server at localhost:{}", port);

    let listener = TcpListener::bind(format!("127.0.0.1:{}",port)).unwrap();

    let pool = ThreadPool::new(4).unwrap();
    
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            read_request(stream);
        });
    }
}

fn read_request(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();

    println!("Request received {}", String::from_utf8(buffer.to_vec()).unwrap());

    send_response(stream);
}

fn send_response(mut stream: TcpStream) {
    let response = "HTTP/1.1 200 Ok\n\n<html><body>Hello, World!</body></html>";

    stream.write_all(response.as_bytes()).unwrap();
}