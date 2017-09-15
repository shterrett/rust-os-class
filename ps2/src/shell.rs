use std::io::{ self, Write };
use std::error::Error;
use std::path::PathBuf;
use program::{
    Program,
    resolve_program
};
use history::History;
use cmd_line::{ CmdLine, ParsedCommand, parse_command };
use external;
use builtin;


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
        match parse_command(program) {
            Ok(ParsedCommand::SingleCommand(cmd_line)) => self.execute(cmd_line),
            Ok(ParsedCommand::PipeChain(cmds)) => self.execute_chain(cmds),
            Err(e) => println!("invalid command: {}", e)
        };
    }

    pub fn execute_chain(&mut self, cmds: Vec<CmdLine>) {
        let invalid = cmds.iter().filter_map(|cmd| {
            match resolve_program(cmd.clone()) {
                Program::NotFound(name) => Some(Program::NotFound(name)),
                Program::Internal(cmd) => Some(Program::Internal(cmd)),
                Program::External(_) => None
            }
        }).collect::<Vec<Program>>();

        if !invalid.is_empty() {
            println!("invalid pipe chain");
            for cmd in invalid {
                match cmd {
                    Program::NotFound(name) => println!("Not found: {}", name),
                    Program::Internal(cmd) => println!("Not pipeable: {}", cmd.name),
                    Program::External(_) => ()
                }
            }
        } else {
            let child = external::run_chain(&cmds, self);
            if let Err(e) = child.and_then(|mut c| c.wait().map_err(|e| e.description().to_string())).map(|_| ()) {
                println!("{}", e);
            }
        }
    }

    pub fn execute(&mut self, cmd_line: CmdLine) {
        let result = match resolve_program(cmd_line) {
            Program::NotFound(name) => {
                Err(format!("{} not found", name))
            },
            Program::Internal(cmd) => {
                builtin::run(&cmd, self)
            },
            Program::External(cmd) => {
                let child = external::run(&cmd, self);
                if !cmd.background {
                    child.and_then(|mut c| c.wait().map_err(|e| e.description().to_string())).map(|_| ())
                } else {
                    child.map(|_| ())
                }
            }
        };

        if let Err(e) = result {
            println!("{}", e);
        }
    }
}
