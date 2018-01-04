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

use std::io::Read;
use std::net::{ TcpListener, TcpStream };
use std::str;
use std::thread;
use std::sync::{ Arc, Mutex };
use std::sync::atomic::{ AtomicUsize, Ordering };

mod path;
mod handler;
mod http;
mod shell_interpolation;
mod cmd_line;
mod external;
mod scheduling;

use scheduling::{ schedule, queues, FastLane, SlowLane };

fn main() {
    let addr = "127.0.0.1:4414";
    let listener = TcpListener::bind(addr).unwrap();
    let visitor_count: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let (hq, lq) = queues();
    let high_priority: Arc<Mutex<FastLane>> = Arc::new(Mutex::new(hq));
    let low_priority: Arc<Mutex<SlowLane>> = Arc::new(Mutex::new(lq));

    println!("Listening on [{}] ...", addr);

    for _ in 1..4 {
        let count = visitor_count.clone();
        let high = high_priority.clone();
        thread::spawn(move || {
            loop {
                let mut queue = high.lock().unwrap();
                if let Some(stream) = queue.pop().map(|s| s.stream) {
                    handle_incoming(stream, count.load(Ordering::Relaxed));
                } else {
                    thread::yield_now();
                }
            }
        });
    }
    for _ in 1..2 {
        let count = visitor_count.clone();
        let low = low_priority.clone();
        thread::spawn(move || {
            loop {
                let mut queue = low.lock().unwrap();
                if let Some(stream) = queue.pop().map(|s| s.stream) {
                    handle_incoming(stream, count.load(Ordering::Relaxed));
                } else {
                    thread::yield_now();
                }
            }
        });
    }

    for stream in listener.incoming() {
        match stream {
            Err(_) => (),
            Ok(stream) => {
                safe_increment(&visitor_count);

                let mut high_queue = high_priority.lock().unwrap();
                let mut low_queue = low_priority.lock().unwrap();
                schedule(stream, &mut high_queue, &mut low_queue)
            }
        }
    }

    drop(listener);
}

fn handle_incoming(mut stream: TcpStream, visitor_count: usize) {
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
}

fn safe_increment(visitor_count: &Arc<AtomicUsize>) {
    let current = visitor_count.load(Ordering::Relaxed);
    let changed = visitor_count.compare_and_swap(current, current + 1, Ordering::Relaxed);
    if current == changed { return; } else { safe_increment(visitor_count) }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;
    use std::sync::atomic::{ AtomicUsize, Ordering };
    use std::thread;

    use super::{
        safe_increment
    };

    #[test]
    fn test_safe_increment() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut threads = Vec::new();

        for _ in 1..5 {
            let local_counter = counter.clone();
            let thread = thread::spawn(move || {
                safe_increment(&local_counter);
            });
            threads.push(thread);
        }

        for thread in threads {
            thread.join().unwrap();
        }

        assert_eq!(counter.load(Ordering::Relaxed), 4);
    }
}
