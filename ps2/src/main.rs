//
// gash.rs
//
// Starting code for PS2
// Running on Rust 1+
//
// University of Virginia - cs4414 Spring 2014
// Weilin Xu, David Evans
// Version 0.4
//

#[macro_use]
extern crate lazy_static;
extern crate getopts;
extern crate chan_signal;

use getopts::Options;
use std::env;
use std::process::{ exit, Child };
use std::sync::{ Arc, Mutex };
use std::thread;
use chan_signal::Signal;

mod program;
mod builtin;
mod shell;
use shell::Shell;
mod history;
mod cmd_line;
mod external;

fn get_cmdline_from_args() -> Option<String> {
    /* Begin processing program arguments and initiate the parameters. */
    let args: Vec<_> = env::args().collect();

    let mut opts = Options::new();
    opts.optopt("c", "", "", "");

    opts.parse(&args[1..]).unwrap().opt_str("c")
}

fn main() {
    let running_child: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));

    if let Ok(working_dir) = std::env::current_dir() {
        let opt_cmd_line = get_cmdline_from_args();
        match opt_cmd_line {
            Some(cmd_line) => Shell::new("", working_dir, Arc::clone(&running_child)).run_program(&cmd_line),
            None => {
                let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);
                let sig_target = Arc::clone(&running_child);
                thread::spawn(move || {
                    loop {
                        if let Some(_) = signal.recv() {
                            if let Err(_) = sig_target.lock().map(|mut running| {
                                if let Some(ref mut current_cmd) = *running {
                                    if let Err(_) = current_cmd.kill() {
                                        println!("Unable to comply with kill command");
                                    }
                                }
                            }) {
                                println!("Unable to comply with kill command");
                            };
                        }
                    }
                });
                let shell_target = Arc::clone(&running_child);
                let child = thread::spawn(move || {
                    Shell::new("gash > ", working_dir, shell_target).run()
                });

                match child.join() {
                    Ok(_) => exit(0),
                    Err(_) => exit(1)
                }
            }
        }
    } else {
        exit(1);
    }
}
