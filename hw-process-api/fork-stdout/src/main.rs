extern crate nix;
use nix::unistd::{ fork, ForkResult, close };
use nix::sys::wait::{ waitpid, WaitStatus };

fn main() {
    match fork() {
        Ok(ForkResult::Parent{child, ..}) => {
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
            let stdout_fd = 1;
            match close(stdout_fd) {
                Ok(_) => println!("stdout closed - Does this even work?"),
                Err(_) => println!("closing stdout failed.")
            }
        },
        Err(_) => {
            println!("Fork failed");
        }
    }
}
