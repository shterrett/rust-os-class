// Copied with modification from ps2

use std::str::Split;
use std::path::Path;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum CmdIO<'a> {
   Console,
   File(&'a Path),
   Pipe
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct CmdLine<'a> {
    pub name: &'a str,
    pub args: Vec<&'a str>,
    pub background: bool,
    pub stdin: CmdIO<'a>,
    pub stdout: CmdIO<'a>
}

pub enum ParsedCommand<'a> {
    SingleCommand(CmdLine<'a>),
    PipeChain(Vec<CmdLine<'a>>)
}

pub fn parse_command<'a>(cmd: &'a str) -> Result<ParsedCommand<'a>, String> {
    if cmd.contains("|") {
        let piped_commands = cmd.split("|")
                                    .map(|c| CmdLine::parse(c))
                                    .collect::<Vec<Result<CmdLine, String>>>();
        lift_result(piped_commands)
            .map(|cmds| set_pipe_io(cmds))
            .map(|cs| ParsedCommand::PipeChain(cs))
    } else {
        CmdLine::parse(cmd).map(|c| ParsedCommand::SingleCommand(c))
    }

}

impl<'a> CmdLine<'a> {
    fn empty() -> Self {
        CmdLine {
            name: "",
            args: vec![],
            background: false,
            stdin: CmdIO::Console,
            stdout: CmdIO::Console
        }
    }
    fn add_arg(mut self, arg: &'a str) -> Self {
        self.args.push(arg);
        self
    }
    fn name(self, name: &'a str) -> Self {
        CmdLine {
            name: name,
            args: self.args,
            background: self.background,
            stdin: self.stdin,
            stdout: self.stdout
        }
    }
    fn background(self, background: bool) -> Self {
        CmdLine {
            name: self.name,
            args: self.args,
            background: background,
            stdin: self.stdin,
            stdout: self.stdout
        }
    }
    fn stdin(self, stdin: CmdIO<'a>) -> Self {
        CmdLine {
            name: self.name,
            args: self.args,
            background: self.background,
            stdin: stdin,
            stdout: self.stdout
        }
    }
    fn stdout(self, stdout: CmdIO<'a>) -> Self {
        CmdLine {
            name: self.name,
            args: self.args,
            background: self.background,
            stdin: self.stdin,
            stdout: stdout
        }
    }

    pub fn parse(line: &'a str) -> Result<Self, String> {
        let words = line
            .trim()
            .split(" ");

        parse_phrase(words, CmdLine::empty())
    }
}

fn parse_phrase<'a>(mut words: Split<'a, &str>, cmd: CmdLine<'a>) -> Result<CmdLine<'a>, String> {
    let next_word = words.next();
    match next_word {
        Some(">") => {
            words.next()
                .ok_or("must provide target for stdout".to_string())
                .map(|s| CmdIO::File(Path::new(s)))
                .and_then(|stdout| parse_phrase(words, cmd.stdout(stdout)))
        },
        Some("<") => {
            words.next()
                .ok_or("must provide target for stdin".to_string())
                .and_then(|s| resolve_stdin(s))
                .and_then(|stdin| parse_phrase(words, cmd.stdin(stdin)))
        },
        Some("&") => {
            parse_phrase(words, cmd.background(true))
        },
        Some(arg) => {
            if cmd.name == "" {
                parse_phrase(words, cmd.name(arg))
            } else {
                parse_phrase(words, cmd.add_arg(arg))
            }
        },
        None => {
            Ok(cmd)
        }
    }
}

fn resolve_stdin(input: &str) -> Result<CmdIO, String> {
    let path = Path::new(input);
    if  path.is_file() {
        Ok(CmdIO::File(&path))
    } else {
        Err(format!("{} is not a valid file", input))
    }
}

fn lift_result<T, E>(results: Vec<Result<T, E>>) -> Result<Vec<T>, E> {
    results.into_iter().fold(Ok(vec![]), |acc, res| {
        match (acc, res) {
            (Ok(mut v), Ok(t)) => {
                v.push(t);
                Ok(v)
            },
            (Ok(_), Err(e)) => Err(e),
            (Err(e), _) => Err(e)
        }
    })
}

fn set_pipe_io(piped_commands: Vec<CmdLine>) -> Vec<CmdLine> {
    let first_idx = 0;
    let last_idx = piped_commands.iter().len() - 1;
    piped_commands
        .into_iter()
        .enumerate()
        .map(|(idx, cmd)| set_io(cmd, idx, first_idx, last_idx))
        .collect::<Vec<CmdLine>>()
}

fn set_io(cmd: CmdLine, idx: usize, first_idx: usize, last_idx: usize) -> CmdLine {
    if first_idx == idx {
        cmd.stdout(CmdIO::Pipe)
    } else if last_idx == idx {
        cmd.stdin(CmdIO::Pipe)
    } else {
        cmd.stdout(CmdIO::Pipe)
            .stdin(CmdIO::Pipe)
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;
    use std::fs::{ File, remove_file };
    use super::{
        CmdLine,
        CmdIO,
        ParsedCommand,
        parse_command,
        lift_result
    };

    #[test]
    fn splits_a_string_into_command_and_arguments() {
        let command = "ls -l";

        match CmdLine::parse(command) {
            Ok(cmd_line) => {
                assert_eq!(cmd_line.name, "ls");
                assert_eq!(cmd_line.args, vec!["-l"]);
                assert_eq!(cmd_line.background, false);
            },
            Err(e) => {
                assert!(false, "parse failed with {}", e);
            }
        }
    }

    #[test]
    fn marks_cmd_line_as_background_task_when_ends_in_ampersand() {
        let command = "ls -l &";

        match CmdLine::parse(command) {
            Ok(cmd_line) => {
                assert_eq!(cmd_line.name, "ls");
                assert_eq!(cmd_line.args, vec!["-l"]);
                assert!(cmd_line.background);
            },
            Err(e) => {
                assert!(false, "parse failed with {}", e);
            }
        }
    }

    #[test]
    fn adds_stdout_redirect_when_present() {
        let command = "cat > temp.txt";

        match CmdLine::parse(command) {
            Ok(cmd_line) => {
                assert_eq!(cmd_line.name, "cat");
                assert!(cmd_line.args.is_empty());
                assert_eq!(cmd_line.stdout, CmdIO::File(Path::new("temp.txt")));
                assert_eq!(cmd_line.stdin, CmdIO::Console);
            },
            Err(e) => {
                assert!(false, "parse failed with {}", e);
            }
        }
    }

    #[test]
    fn adds_stdin_redirect_when_present() {
        let _ = File::create("temp.txt").unwrap();
        let command = "cat < temp.txt";

        match CmdLine::parse(command) {
            Ok(cmd_line) => {
                assert_eq!(cmd_line.name, "cat");
                assert!(cmd_line.args.is_empty());
                assert_eq!(cmd_line.stdout, CmdIO::Console);
                assert_eq!(cmd_line.stdin, CmdIO::File(Path::new("temp.txt")));
            },
            Err(e) => {
                assert!(false, "parse failed with {}", e);
            }
        }

        let _ = remove_file(Path::new("temp.txt"));
    }

    #[test]
    fn parsing_fails_when_stdin_not_present() {
        let command = "cat < temp.txt";

        match CmdLine::parse(command) {
            Ok(_) => {
                assert!(false, "parse should fail without temp.txt");
            },
            Err(e) => {
                assert_eq!(e, "temp.txt is not a valid file");
            }
        }
    }

    #[test]
    fn test_lift_result() {
        let successful: Vec<Result<usize, &str>> = vec![Ok(1), Ok(2), Ok(3), Ok(4)];
        let success_result = lift_result(successful);
        assert_eq!(success_result, Ok(vec![1, 2, 3, 4]));

        let failure = vec![Ok(1), Err("I'm sorry"), Ok(2), Err("Poor thing")];
        let failure_result = lift_result(failure);
        assert_eq!(failure_result, Err("I'm sorry"));
    }

    #[test]
    fn parse_command_returns_single_command() {
        match parse_command("ls -l") {
            Ok(ParsedCommand::SingleCommand(cmd_line)) => {
                assert_eq!(cmd_line.name, "ls");
                assert_eq!(cmd_line.args, vec!["-l"]);
            },
            Ok(ParsedCommand::PipeChain(_)) => {
                assert!(false, "Single command returned pipe chain");
            },
            Err(e) => assert!(false, e)
        }
    }

    #[test]
    fn parse_command_returns_pipe_chain() {
        match parse_command("ls -l | wc -l") {
            Ok(ParsedCommand::SingleCommand(_)) => {
                assert!(false, "Pipe chain returned singlec ommand");
            },
            Ok(ParsedCommand::PipeChain(cmds)) => {
                let names = cmds.iter().map(|c| c.name).collect::<Vec<&str>>();
                assert_eq!(names, vec!["ls", "wc"]);
                let args = cmds.iter().map(|c| &c.args).collect::<Vec<&Vec<&str>>>();
                assert_eq!(args, vec![&vec!["-l"], &vec!["-l"]]);
            },
            Err(e) => assert!(false, e)
        }
    }

    #[test]
    fn parse_command_sets_std_io_to_pipe_for_internal_commands() {
        match parse_command("ls -l | wc -l") {
            Ok(ParsedCommand::SingleCommand(_)) => {
                assert!(false, "Pipe chain returned singlec ommand");
            },
            Ok(ParsedCommand::PipeChain(cmds)) => {
                let names = cmds.iter().map(|c| &c.stdin).collect::<Vec<&CmdIO>>();
                assert_eq!(names, vec![&CmdIO::Console, &CmdIO::Pipe]);
                let names = cmds.iter().map(|c| &c.stdout).collect::<Vec<&CmdIO>>();
                assert_eq!(names, vec![&CmdIO::Pipe, &CmdIO::Console]);
            },
            Err(e) => assert!(false, e)
        }
    }
}
