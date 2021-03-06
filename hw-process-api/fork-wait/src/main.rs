extern crate nix;
use nix::unistd::{ fork, ForkResult };
use nix::sys::wait::{ waitpid, WaitStatus };

fn main() {
    match fork() {
        Ok(ForkResult::Parent{child, ..}) => {
            println!("Parent spawned!");
            match waitpid(child, None) {
                Err(_) => println!("Wait failed!"),
                Ok(WaitStatus::Exited(_, _)) => println!("Child Exited"),
                Ok(WaitStatus::Signaled(_, _, _)) => println!("Child Signaled"),
                Ok(WaitStatus::Stopped(_, _)) => println!("Child Stopped"),
                Ok(WaitStatus::Continued(_)) => println!("Child Continued"),
                Ok(WaitStatus::StillAlive) => println!("Still Alive?")
            }
        },
        Ok(ForkResult::Child) => {
            println!("Child Executing!");
        },
        Err(_) => {
            println!("Fork failed");
        }
    }
}
