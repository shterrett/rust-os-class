use std::process::Command;
use builtin::{ builtin_exists, Builtin };
use cmd_line::CmdLine;

pub enum Program<'a> {
    Internal(Builtin<'a>),
    External((&'a str, Vec<&'a str>)),
    NotFound(&'a str)
}

pub fn resolve_program<'a>(cmd_line: CmdLine<'a>) -> Program<'a> {
    if builtin_exists(cmd_line.name) {
        Program::Internal(
            Builtin::new(cmd_line.name, cmd_line.args)
        )
    } else if cmd_exists(cmd_line.name) {
        Program::External((cmd_line.name, cmd_line.args))
    } else {
        Program::NotFound(cmd_line.name)
    }
}

fn cmd_exists(cmd_path: &str) -> bool {
    Command::new("which").arg(cmd_path).status().unwrap().success()
}

#[cfg(test)]
mod test {
    use builtin::Builtin;
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
        let program = resolve_program(command);

        match program {
            Program::Internal(builtin) => {
                assert_eq!(builtin, Builtin::new("cd", vec!["/usr/bin"]));
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
            Program::External((cmd, args)) => {
                assert_eq!(cmd, "ls");
                assert_eq!(args, vec!["-al"]);
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
