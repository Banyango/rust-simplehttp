use notify::{Watcher, RecursiveMode, watcher,RecommendedWatcher};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use std::thread;
use colored::*;
use websocket::sync::{Server, Client};
use websocket::OwnedMessage;
use bus::Bus;

#[derive(Clone)]
pub enum FileEvent {
    FilesChange,
    None,
}

pub struct FileWatcher {
    watcher:RecommendedWatcher,
    file_thread:Option<std::thread::JoinHandle<()>>,
    client_thread:Option<std::thread::JoinHandle<()>>,
}

impl FileWatcher {
    pub fn new() -> Self {
        let mut bus = Arc::new(Mutex::new(Bus::new(10)));
        
        let (tx, rx) = mpsc::channel();
       
        let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();

        watcher.watch(std::env::current_dir().unwrap(), RecursiveMode::Recursive).unwrap();        

        let own_bus = bus.clone();
        let file_thread = Some(std::thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(event) => {                        
                        own_bus.lock().unwrap().broadcast(FileEvent::FilesChange);
                    },
                    Err(e) => println!("watch error: {:?}", e),
                }
            }
        }));

        let client_thread = Some(std::thread::spawn(move || {
                
                println!("{}","Starting hot reload server...".yellow());
                
                let server = Server::bind("127.0.0.1:30012").unwrap(); 

                for request in server.filter_map(Result::ok) {                    
                    let client_bus = bus.clone();
                    thread::spawn(move || {                                                
                        let mut client = request.use_protocol("rust-simplehttp").accept().unwrap();
                        
                        client.set_nonblocking(true).unwrap();

                        let mut rec = client_bus.lock().unwrap().add_rx();                        
                        loop {
                            match client.recv_message() {
                                Ok(OwnedMessage::Close(_)) => {
                                    return;
                                },
                                _ => {}
                            }
                            match rec.recv() {
                                Ok(FileEvent::FilesChange) => {                                    
                                    let message = OwnedMessage::Text("Files Changed".to_string());                                    
			                        client.send_message(&message).unwrap(); 
                                },
                                Ok(_)=> {},                               
                                Err(e)=> { return; }
                            }
                        }
                        
                    });

                                      
                }
                                      
        }));
        
        FileWatcher {
            watcher,
            file_thread,
            client_thread
        }    
    }
}