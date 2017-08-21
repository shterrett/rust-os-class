use std::process::Command;
use builtin::{ builtin_exists, Builtin };

pub enum Program<'a> {
    Internal(Builtin<'a>),
    External((&'a str, Vec<&'a str>)),
    NotFound
}

pub fn resolve_program<'a>(cmd: &'a str) -> Program<'a> {
    if let  Some((cmd_path, args)) = cmd_and_args(cmd) {
        if builtin_exists(cmd_path) {
            Program::Internal(
                Builtin::new(cmd_path, args)
            )
        } else if cmd_exists(cmd_path) {
            Program::External((cmd_path, args))
        } else {
            Program::NotFound
        }
    } else {
        Program::NotFound
    }
}

fn cmd_and_args<'a>(cmd: &'a str) -> Option<(&'a str, Vec<&'a str>)> {
    let argv: Vec<&str> = cmd.split(' ').filter_map(|x| {
        if x == "" {
            None
        } else {
            Some(x)
        }
    }).collect();

    argv.split_first()
        .map(|(&name, args)| (name, args.to_vec()))
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
        cmd_and_args,
        resolve_program
    };

    #[test]
    fn splits_a_string_into_command_and_arguments() {
        let command = "ls -l";

        assert_eq!(cmd_and_args(command), Some(("ls", vec!["-l"])));
    }

    #[test]
    // This test assumes `ls` exists
    // and that `not-a-command` does not
    fn determines_if_command_exists() {
        assert!(cmd_exists("ls"));
        assert!(!cmd_exists("not-a-command"));
    }

    #[test]
    fn returns_the_builtin_program() {
        let command = "cd /usr/bin";
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
        let command = "ls -al";
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
    // This test assumez `not-a-command` does not exist
    fn returns_not_found_for_nonexistant_command() {
        let command = "not-a-command";
        let program = resolve_program(command);

        match program {
            Program::NotFound =>  assert!(true),
            _ => assert!(false, "wrong program type")
        }
    }
}
