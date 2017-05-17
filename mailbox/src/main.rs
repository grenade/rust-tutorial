extern crate redisish;
use redisish::{parse, Command, Error};
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::collections::VecDeque;
use std::thread;
use std::sync::{Arc, Mutex};

fn main() {
  //fn handle_request<S: Storage<String>>(stream: &mut TcpStream, storage: &S) {
  fn handle_request<S: Storage<String>>(stream: &mut TcpStream, storage: Arc<S>) {
    let mut buffer = String::new();
    stream.read_to_string(&mut buffer);
    let result = parse(&buffer).unwrap();
    match result {
      Command::Publish(message) => storage.put(message),
      Command::Retrieve => {
        match stream.write(storage.get().unwrap().as_bytes()) {
          Ok(_) => println!("Response sent."),
          Err(e) => println!("Failed sending response: {}", e),
        }
      },
    }
  }
  let arced_mailbox: Arc<SyncedMailbox<String>> = Arc::new(SyncedMailbox::new());
  let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
  for stream in listener.incoming() {
    let cloned_arc = arced_mailbox.clone();
    thread::spawn(move || match stream {
      Ok(mut stream) => {
        //handle_request(&mut stream, cloned_arc.as_ref());
        handle_request(&mut stream, cloned_arc);
        println!("Request received.")
      },
      Err(e) => println!("Failed receiving request: {}", e),
    });
  }
}

struct SyncedMailbox<T> {
  all_the_mail: Mutex<VecDeque<T>>,
}

impl<T> SyncedMailbox<T> {
  fn new () -> SyncedMailbox<T> {
    SyncedMailbox {
      all_the_mail: Mutex::new(VecDeque::new())
    }
  }
}

trait Storage<T> {
  fn put(&self, item: T) -> ();
  fn get(&self) -> Option<T>;
}

impl<T> Storage<T> for SyncedMailbox<T> {
  fn put(&self, item: T) {
    self.all_the_mail.lock().unwrap().push_back(item);
  }
  fn get(&self) -> Option<T> {
    self.all_the_mail.lock().unwrap().pop_back()
  }
}