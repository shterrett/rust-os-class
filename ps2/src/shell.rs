use std::io::{ self, Write };
use std::path::PathBuf;
use program::{
    Program,
    resolve_program
};
use history::History;
use cmd_line::CmdLine;
use external::run_external;


pub struct Shell<'a> {
    cmd_prompt: &'a str,
    pub working_dir: PathBuf,
    pub history: History
}

impl<'a> Shell<'a> {
    pub fn new(prompt_str: &'a str, path: PathBuf) -> Shell<'a> {
        Shell {
            cmd_prompt: prompt_str,
            working_dir: path,
            history: History::new(100)
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
            {
                let cmd_line = line.trim();
                let program = cmd_line.splitn(1, ' ').nth(0).expect("no program");

                self.run_program(program);
            }
            self.history.add(line);
        }
    }

    pub fn run_program(&mut self, program: &str) {
        match CmdLine::parse(program) {
            Some(cmd_line) => self.execute(cmd_line),
            None => println!("invalid command")
        };
    }

    pub fn execute(&mut self, cmd_line: CmdLine) {
        let background = cmd_line.background;

        match resolve_program(cmd_line) {
            Program::NotFound(name) => {
                println!("{} not found", name);
            },
            Program::Internal(builtin) => {
                builtin.run(self);
            },
            Program::External((cmd_path, args)) => {
                run_external(self, cmd_path, args, background);
            }
        }
    }
}
