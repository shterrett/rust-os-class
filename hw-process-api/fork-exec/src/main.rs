use std::ffi::CString;

extern crate nix;
use nix::unistd::{
    fork,
    ForkResult,
    execv
};

fn main() {
    match fork() {
        Ok(ForkResult::Parent{..}) => {
            println!("Running ls with execv");
        },
        Ok(ForkResult::Child) => {
            let ls = CString::new("/bin/ls").unwrap();
            let args = [CString::new("-l").unwrap()];
            match execv(&ls, &args) {
                Ok(_) => println!("Executed successfully"),
                Err(_) => println!("Execution failed")
            }
        },
        Err(_) => {
            println!("Fork failed");
        }
    }
}
