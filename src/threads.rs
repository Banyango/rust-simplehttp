use std::thread;
use std::sync::{mpsc, Arc, Mutex};
use std::error::Error;
use std::fmt;

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

type Job = Box<FnBox + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Result<Self, PoolCreationError> {
        
        if size == 0 {
            return Err(PoolCreationError);
        }

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for i in 0..size {
            workers.push(Worker::new(i, receiver.clone()))
        }

        Ok(ThreadPool { workers, sender})
    }

    pub fn execute<F>(&self, f:F) where F : FnOnce() + Send + 'static {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in 0..self.workers.len() {
            self.sender.send(Message::Terminate).unwrap();
        }
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

pub struct Worker {
    id:usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id:usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    job.call_box();
                },
                Message::Terminate => {
                    break;
                }
            }
        });

        Worker {
            id:id,
            thread:Some(thread)
        }
    }
}

//--- Pool Creation Error ----
#[derive(Debug)]
pub struct PoolCreationError;

impl Error for PoolCreationError {
    fn description(&self) -> &str {
        "Thread size cannot be zero"
    }
}

impl fmt::Display for PoolCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PoolCreationError: ThreadPool's `size` can't be zero")
    }
}