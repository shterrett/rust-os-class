//
// zhttpto.rs
//
// Starting code for PS1
// Running on Rust 1+
//
// Note that this code has serious security risks! You should not run it
// on any system with access to sensitive files.
//
// University of Virginia - cs4414 Spring 2014
// Weilin Xu and David Evans
// Version 0.3

#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::io::{Read};
use std::net::TcpListener;
use std::str;
use std::thread;

mod path;
mod handler;
mod http;

fn main() {
    let addr = "127.0.0.1:4414";

    let listener = TcpListener::bind(addr).unwrap();

    let mut visitor_count: u16 = 0;

    println!("Listening on [{}] ...", addr);

    for stream in listener.incoming() {
        match stream {
            Err(_) => (),
            Ok(mut stream) => {
                visitor_count += 1;

                // Spawn a thread to handle the connection
                thread::spawn(move|| {
                    match stream.peer_addr() {
                        Err(_) => (),
                        Ok(pn) => println!("Received connection from: [{}]", pn),
                    }

                    let mut buf = [0 ;500];
                    stream.read(&mut buf).unwrap();
                    match str::from_utf8(&buf) {
                        Err(error) => println!("Received request error:\n{}", error),
                        Ok(body) => {
                            let req_path = path::path(body);
                            println!("Recieved request body:\n{}", body);
                            println!("Requested Path: {}\n", req_path);
                            let status = handler::handle_request(req_path, visitor_count, &mut stream);
                            println!("Response Status: {}", status);
                        }
                    }

                    println!("Connection terminates.");
                });
            },
        }
    }

    drop(listener);
}
