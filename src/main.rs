extern crate clap;

use clap::{Arg, App};

use std::io::{Read, Write, BufReader, BufRead};
use std::env;
use std::net::{TcpListener, TcpStream};

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

    let listener = TcpListener::bind(format!("127.0.0.1:{}",matches.value_of("port").unwrap_or("3000"))).unwrap();

    let stream = listener.accept().unwrap().0;

    read_request(stream);
}

fn read_request(stream: TcpStream) {
    let mut reader = BufReader::new(stream);

    for line in reader.by_ref().lines() {
        if line.unwrap() == "" {
            break;
        }    
    }

    send_response(reader.into_inner());
}

fn send_response(mut stream: TcpStream) {
    let response = "HTTP/1.1 200 Ok\n\n<html><body>Hello, World!</body></html>";

    stream.write_all(response.as_bytes()).unwrap();
}