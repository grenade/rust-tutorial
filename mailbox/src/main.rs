extern crate redisish;
use redisish::{parse, Command, Error};
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::collections::VecDeque;
use std::thread;
use std::sync::{Arc, Mutex};

fn main() {
  fn handle_client(mut stream: TcpStream, messages: Arc<Mutex<VecDeque<String>>>) {
    let mut buffer = String::new();
    stream.read_to_string(&mut buffer);
    let result = parse(&buffer).unwrap();
    match result {
      Command::Publish(message) => messages.lock().unwrap().push_back(message),
      Command::Retrieve => {
        match stream.write(messages.lock().unwrap().pop_back().unwrap().as_bytes()) {
          Ok(_) => println!("Response sent"),
          Err(e) => println!("Failed sending response: {}", e),
        }
      }
    }
  }
  let messages = Arc::new(Mutex::new(VecDeque::with_capacity(10)));
  let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
  for stream in listener.incoming() {
    let storage = messages.clone();
    thread::spawn(move || match stream {
      Ok(stream) => handle_client(stream, storage),
      Err(e) => println!("Failed receiving request: {}", e),
    });
  }
}
