extern crate clap;
#[macro_use]
extern crate nom;

mod threads;
mod parser;

use clap::{Arg, App};

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::fs::File;


use crate::threads::ThreadPool;
use crate::parser::*;

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

    match parse_request(&buffer) {
        Ok(result) => {
            match result.method {
                Method::Get => {                                                            
                    let path = &result.uri.as_path();

                    println!("GET request on path {}", path.display());

                    match File::open(&path) {
                        Ok(mut file) => { send_response(stream, &mut file); },
                        Err(e) => { 
                            println!("{}",e);
                            send_error_response(stream, "404 Not Found", Some("<html><body>404 page not found</body></html>"));},
                    };                    
                },
                // impl HEAD
                _ => send_error_response(stream, "405 Method Not Allowed", None)
            }
        },
        Err(_e) => {
            send_error_response(stream, "500 Internal Server Error", None)
        }
    };

}

fn send_error_response(mut stream: TcpStream, code:&str, body:Option<&str>) {    
    println!("Server error {}", code);

    let response = "HTTP/1.1 ".to_owned() + code + "\n\n" + body.unwrap_or("");

    stream.write_all(response.as_bytes()).unwrap();
}

fn send_response(mut stream: TcpStream, file: &mut File) {

    let mut file_contents = Vec::new();
        
    file.read_to_end(&mut file_contents).unwrap();
    println!("Sending Response {:#?}", String::from_utf8(file_contents.clone()).unwrap());

    let response = ["HTTP/1.1 200 Ok\n\n".as_bytes(), &file_contents].concat();   

    stream.write_all(&response).unwrap();

}