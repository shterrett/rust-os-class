use std::process::{ Command, Stdio };
use builtin::builtin_exists;
use cmd_line::CmdLine;

pub enum Program<'a> {
    Internal(CmdLine<'a>),
    External(CmdLine<'a>),
    NotFound(&'a str)
}

pub fn resolve_program<'a>(cmd_line: CmdLine<'a>) -> Program<'a> {
    if builtin_exists(cmd_line.name) {
        Program::Internal(cmd_line)
    } else if cmd_exists(cmd_line.name) {
        Program::External(cmd_line)
    } else {
        Program::NotFound(cmd_line.name)
    }
}

fn cmd_exists(cmd_path: &str) -> bool {
    Command::new("which")
        .arg(cmd_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .status()
        .unwrap()
        .success()
}

#[cfg(test)]
mod test {
    use super::{
        Program,
        cmd_exists,
        resolve_program
    };
    use cmd_line::CmdLine;

    #[test]
    // This test assumes `ls` exists
    // and that `not-a-command` does not
    fn determines_if_command_exists() {
        assert!(cmd_exists("ls"));
        assert!(!cmd_exists("not-a-command"));
    }

    #[test]
    fn returns_the_builtin_program() {
        let command = CmdLine::parse("cd /usr/bin").unwrap();
        let program = resolve_program(command.clone());

        match program {
            Program::Internal(builtin) => {
                assert_eq!(builtin, command);
            },
            _ => {
                assert!(false, "wrong program type");
            }
        }
    }

    #[test]
    // This test assumes `ls` exists
    fn returns_external_program() {
        let command = CmdLine::parse("ls -al").unwrap();
        let program = resolve_program(command);

        match program {
            Program::External(cmd) => {
                assert_eq!(cmd.name, "ls");
                assert_eq!(cmd.args, vec!["-al"]);
            },
            _ => {
                assert!(false, "wrong program type");
            }
        }
    }

    #[test]
    // This test assumes `not-a-command` does not exist
    fn returns_not_found_for_nonexistant_command() {
        let command = CmdLine::parse("not-a-command").unwrap();
        let program = resolve_program(command);

        match program {
            Program::NotFound(_) =>  assert!(true),
            _ => assert!(false, "wrong program type")
        }
    }
}
