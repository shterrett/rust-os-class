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

use getopts::Options;
use std::env;
use std::process::exit;

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
    if let Ok(working_dir) = std::env::current_dir() {
        let opt_cmd_line = get_cmdline_from_args();
        match opt_cmd_line {
            Some(cmd_line) => Shell::new("", working_dir).run_program(&cmd_line),
            None           => Shell::new("gash > ", working_dir).run(),
        }
    } else {
        exit(1);
    }
}
