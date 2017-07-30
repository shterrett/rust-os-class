extern crate nix;

use nix::fcntl::{ open, O_WRONLY };
use nix::sys::stat::Mode;
use nix::unistd::{ fork, ForkResult, write };

fn main() {
    let mode = Mode::empty();
    let fd = open("./test.txt", O_WRONLY, mode).unwrap();

    match fork() {
        Ok(ForkResult::Parent{..}) => {
            let buf = "Write from parent".as_bytes();
            match write(fd, buf) {
                Ok(_) => println!("Parent wrote!"),
                Err(_) => println!("Parent write failed!")
            }
        },
        Ok(ForkResult::Child) => {
            let buf = "Write from child".as_bytes();
            match write(fd, buf) {
                Ok(_) => println!("Child wrote!"),
                Err(_) => println!("Child write failed!")
            }
        },
        Err(_) => {
            println!("Fork failed!");
        }
    }
}
