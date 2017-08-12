use std::io::{ self, Write };
use std::path::PathBuf;
use std::process::Command;
use program::{
    Program,
    resolve_program
};

pub struct Shell<'a> {
    cmd_prompt: &'a str,
    pub working_dir: PathBuf
}

impl<'a> Shell<'a> {
    pub fn new(prompt_str: &'a str, path: PathBuf) -> Shell<'a> {
        Shell {
            cmd_prompt: prompt_str,
            working_dir: path
        }
    }

    pub fn run(&mut self) {
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

    pub fn run_program(&mut self, program: &str) {
        match resolve_program(program) {
            Program::NotFound => {
                println!("{} not found", program);
            },
            Program::Internal(builtin) => {
                builtin.run(self);
            },
            Program::External((cmd_path, args)) => {
                io::stdout()
                    .write(&Command::new(cmd_path)
                           .args(args)
                           .current_dir(self.working_dir.clone())
                           .output()
                           .unwrap()
                           .stdout)
                    .unwrap();
            }
        }
    }
}