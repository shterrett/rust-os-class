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
