use std::process::{ exit };
extern crate nix;
use nix::unistd::{ fork, ForkResult, dup2, pipe, write, read, close };

fn main() {
    let (parent_read, parent_write) = pipe().unwrap();
    match fork() {
        Ok(ForkResult::Parent{..}) => {
            match fork() {
                Ok(ForkResult::Parent{..}) => {
                },
                Ok(ForkResult::Child) => {
                    let child_write = dup2(1, parent_write).unwrap();
                    let message = "Hello other child".as_bytes();
                    match write(child_write, message) {
                        Ok(_) => println!("Sibling wrote"),
                        Err(_) => println!("Sibling write failed")
                    }
            match close(child_write) {
                Ok(_) => exit(0),
                Err(_) => exit(1)
            }
                },
                Err(_) => {
                    println!("Fork failed!");
                }
            }
        },
        Ok(ForkResult::Child) => {
            let child_read = dup2(0, parent_read).unwrap();
            let mut buff: Vec<u8> = Vec::new();
            match read(child_read, &mut buff) {
                Ok(_) => println!("Read from sibling: {}", String::from_utf8(buff).unwrap()),
                Err(_) => println!("Read failed")
            }
            match close(child_read) {
                Ok(_) => exit(0),
                Err(_) => exit(1)
            }
        },
        Err(_) => {
            println!("Fork failed");
        }
    }
}
