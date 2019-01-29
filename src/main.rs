extern crate clap;
#[macro_use]
extern crate nom;
extern crate colored; 
#[macro_use]
extern crate lazy_static;
    
mod threads;
mod parser;

use colored::*;
use clap::{Arg, App};

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::fs::File;

use std::sync::Mutex;

lazy_static! {
    static ref ServerConfig: Mutex<Config> = Mutex::new(Config{port:"".to_string(), verbose:false});
}

use crate::threads::ThreadPool;
use crate::parser::*;

pub struct Config{
    port:String,
    verbose:bool,
}

fn main() {

    let matches = App::new("Simple Http Server")
        .version("0.1.0")
        .author("Kyle R. <kyle@banyango.com>")
        .about("Sets up a simple http server for serving static content from a directory")
        .arg(Arg::with_name("port")
            .short("p")
            .takes_value(true)
            .long("port")
            .required(false)
            .help("Set the port to serve on"))
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .required(false)
            .help("Set the logging to verbose"))
            .get_matches();    

    let port = matches.value_of("port").unwrap_or("3000");

    let verbose = matches.is_present("verbose");

    ServerConfig.lock().unwrap().port = port.to_string();
    ServerConfig.lock().unwrap().verbose = verbose;

    
    println!("{}","#####################################".blue());
    println!("{}","         rust-simplehttp             ".blue());
    println!("{}","#####################################".blue());
    println!("{} at {}{} verbose {}","Starting server".green(),"localhost:".bold(), port, verbose);
    
    if !port_is_available(&ServerConfig.lock().unwrap().port) {
        println!("{} Could not open listener on port = {}","Error: port is in use".red(), &ServerConfig.lock().unwrap().port);        
    } else {
        let listener = TcpListener::bind(format!("127.0.0.1:{}",port))
            .expect("Unable to start server on requested port");

        println!("{}","Server started!".green());
        
        let pool = ThreadPool::new(4).unwrap();
        
        for stream in listener.incoming() {
            let stream = stream.unwrap();

            pool.execute(|| {
                read_request(stream);
            });
        }
    }
}

fn port_is_available(port: &str) -> bool {
    match TcpListener::bind(format!("127.0.0.1:{}",port)) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn read_request(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();

    if ServerConfig.lock().unwrap().verbose {        
        println!("Request received {}", String::from_utf8(buffer.to_vec()).unwrap().red());
    }

    match parse_request(&buffer) {
        Ok(result) => {
            match result.method {
                Method::Get => {                                                            
                    let path = &result.uri.as_path();

                    println!("GET request {}", result.original_uri);
                    
                    match File::open(&path) {
                        Ok(mut file) => { send_response(stream, &mut file); },
                        Err(e) => { 
                            println!("{}{}","Error:".red(),e);
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
    println!("{}{}","Server Error:".red(),code);

    let response = "HTTP/1.1 ".to_owned() + code + "\n\n" + body.unwrap_or("");

    stream.write_all(response.as_bytes()).unwrap();
}

fn send_response(mut stream: TcpStream, file: &mut File) {

    let mut file_contents = Vec::new();
        
    file.read_to_end(&mut file_contents).unwrap();
    
    if ServerConfig.lock().unwrap().verbose { 
        println!("{} {:#?}","Sending Response".green(), String::from_utf8(file_contents.clone()).unwrap());
    }

    let response = ["HTTP/1.1 200 Ok\n\n".as_bytes(), &file_contents].concat();   

    stream.write_all(&response).unwrap();

}