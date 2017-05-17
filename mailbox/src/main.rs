extern crate clap;
use clap::{Arg, App, SubCommand};
extern crate redisish;
use redisish::{parse, Command, Error};
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::collections::VecDeque;
use std::thread;
use std::sync::{Arc, Mutex};

fn main() {
  let matches = App::new("My Sub-Super Program")
    .version("0.1")
    .author("grenade <grenade@mozilla.com>")
    .about("Does sub-awesome things")
    .arg(Arg::with_name("config")
      .short("c")
      .long("config")
      .value_name("FILE")
      .help("Sets a custom config file")
      .takes_value(true))
    .arg(Arg::with_name("INPUT")
      .help("Sets the input file to use")
      .required(true)
      .index(1))
    .arg(Arg::with_name("v")
      .short("v")
      .multiple(true)
      .help("Sets the level of verbosity"))
    .subcommand(SubCommand::with_name("test")
      .about("controls testing features")
      .version("1.3")
      .author("Someone E. <someone_else@other.com>")
      .arg(Arg::with_name("debug")
        .short("d")
        .help("print debug information verbosely")))
    .get_matches();
  // Gets a value for config if supplied by user, or defaults to "default.conf"
  let config = matches.value_of("config").unwrap_or("default.conf");
  println!("Value for config: {}", config);
  // Calling .unwrap() is safe here because "INPUT" is required (if "INPUT" wasn't
  // required we could have used an 'if let' to conditionally get the value)
  println!("Using input file: {}", matches.value_of("INPUT").unwrap());
  // Vary the output based on how many times the user used the "verbose" flag
  // (i.e. 'myprog -v -v -v' or 'myprog -vvv' vs 'myprog -v'
  match matches.occurrences_of("v") {
    0 => println!("No verbose info"),
    1 => println!("Some verbose info"),
    2 => println!("Tons of verbose info"),
    3 | _ => println!("Don't be crazy"),
  }
  // You can handle information about subcommands by requesting their matches by name
  // (as below), requesting just the name used, or both at the same time
  if let Some(matches) = matches.subcommand_matches("test") {
    if matches.is_present("debug") {
      println!("Printing debug info...");
    } else {
      println!("Printing normally...");
    }
  }

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