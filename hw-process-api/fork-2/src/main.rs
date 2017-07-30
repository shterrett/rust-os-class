use std::{ thread, time };

extern crate nix;
use nix::unistd::{ fork, ForkResult };

fn main() {
    match fork() {
        Ok(ForkResult::Parent{..}) => {
            let sleep_time = time::Duration::from_millis(1000);
            thread::sleep(sleep_time);
            println!("From the parent!");
        },
        Ok(ForkResult::Child) => {
            println!("From the child!");
        },
        Err(_) => {
            println!("Fork failed");
        }
    }
}
