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
use std::io::{self, Write};
use std::process::Command;

mod program;
use program::{
    resolve_program,
    Program
};

struct Shell<'a> {
    cmd_prompt: &'a str,
}

impl<'a> Shell<'a> {
    fn new(prompt_str: &'a str) -> Shell<'a> {
        Shell { cmd_prompt: prompt_str }
    }

    fn run(&self) {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        loop {
            stdout.write(self.cmd_prompt.as_bytes()).unwrap();
            stdout.flush().unwrap();

            let mut line = String::new();

            stdin.read_line(&mut line).unwrap();
            let cmd_line = line.trim();
            let program = cmd_line.splitn(1, ' ').nth(0).expect("no program");

            self.run_program(program)
        }
    }

    fn run_program(&self, program: &'a str) {
        match resolve_program(program) {
            Program::NotFound => {
                println!("{} not found", program);
            },
            Program::Internal(builtin) => {
                builtin.run();
            },
            Program::External((cmd_path, args)) => {
                io::stdout()
                    .write(&Command::new(cmd_path)
                           .args(args)
                           .output()
                            .unwrap()
                            .stdout)
                    .unwrap();
            }
        }
    }
}

fn get_cmdline_from_args() -> Option<String> {
    /* Begin processing program arguments and initiate the parameters. */
    let args: Vec<_> = env::args().collect();

    let mut opts = Options::new();
    opts.optopt("c", "", "", "");

    opts.parse(&args[1..]).unwrap().opt_str("c")
}

fn main() {
    let opt_cmd_line = get_cmdline_from_args();

    match opt_cmd_line {
        Some(cmd_line) => Shell::new("").run_program(&cmd_line),
        None           => Shell::new("gash > ").run(),
    }
}
