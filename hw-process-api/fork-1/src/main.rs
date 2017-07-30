extern crate nix;
use nix::unistd::{ fork, ForkResult };

fn main() {
    let mut state = "Initial";

    match fork() {
        Ok(ForkResult::Parent{..}) => {
            println!("Initial State for Parent: {}", state);
            state = "Parent";
            println!("Final State for Parent: {}", state);
        },
        Ok(ForkResult::Child) => {
            println!("Initial State for Child: {}", state);
            state = "Child";
            println!("Final State for Child: {}", state);
        },
        Err(_) => {
            println!("Fork failed");
        }
    }
}
