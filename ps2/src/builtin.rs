use std::collections::HashSet;
use std::process::exit;
use std::path::{ Path };
use std::env::home_dir;

use shell::Shell;

lazy_static! {
    static ref BUILTINS: HashSet<&'static str> = {
        let mut builtins = HashSet::new();
        builtins.insert("exit");
        builtins.insert("cd");
        builtins.insert("pwd");
        builtins.insert("history");
        builtins
    };
}

pub fn builtin_exists(cmd_path: &str) -> bool {
    BUILTINS.contains(cmd_path)
}


#[derive(PartialEq, Eq, Debug)]
pub struct Builtin<'a> {
    name: &'a str,
    args: Vec<&'a str>
}

impl<'a> Builtin<'a> {
    pub fn new(name: &'a str, args: Vec<&'a str>) -> Self {
        Builtin {
            name: name,
            args: args
        }
    }

    pub fn run(&self, shell: &mut Shell) {
        match self.name {
            "exit" => exit(0),
            "cd" => run_cd(&self.args, shell),
            "pwd" => run_pwd(shell),
            "history" => run_history(&self.args, shell),
            _ => panic!()
        }
    }
}

fn run_cd(args: &Vec<&str>, shell: &mut Shell) {
    let new_path =
        args.first()
            .map(|p| {
                println!("{:?}", p);
                p
            })
            .map(|s| Path::new(s))
            .and_then(|p| {
                if p.is_absolute() {
                    Some(p.to_path_buf())
                } else if p.starts_with("~") {
                    home_dir()
                        .map(|home| {
                             let mut home_rel = p.components();
                             home_rel.next();
                             home.join(
                                 home_rel.as_path()
                             ).to_path_buf()
                        })
                } else {
                    let mut tmp = shell.working_dir.clone();
                    tmp.push(p);
                    Some(tmp)
                }
            })
            .and_then(|p| p.canonicalize().ok())
            .and_then(|p| {
                if p.is_dir() {
                    Some(p)
                } else {
                    None
                }
            });

    if let Some(path) = new_path {
            shell.working_dir = path;
    }
}

fn run_pwd(shell: &Shell) {
    shell.working_dir
        .to_str()
        .map(|dir| println!("{}", dir));
}

fn run_history(args: &Vec<&str>, shell: &Shell) {
    for cmd in shell.history.list(10) {
        println!("{}", cmd);
    }
}

#[cfg(test)]
mod test {
    use super::run_cd;
    use shell::Shell;
    use std::path::Path;
    use std::env::home_dir;

    #[test]
    fn changes_working_directory_to_relative_path() {
        let mut shell = Shell::new(">", Path::new("/").to_path_buf());
        let args = vec!["usr"];

        run_cd(&args, &mut shell);

        assert_eq!(shell.working_dir, Path::new("/usr").to_path_buf());
    }

    #[test]
    fn changes_working_directory_to_absolute_path() {
        let mut shell = Shell::new(">", Path::new("/usr").to_path_buf());
        let args = vec!["/usr/bin"];

        run_cd(&args, &mut shell);

        assert_eq!(shell.working_dir, Path::new("/usr/bin").to_path_buf());
    }

    #[test]
    fn resolves_parent_dir_references() {
        let mut shell = Shell::new(">", Path::new("/usr/bin").to_path_buf());
        let args = vec![".."];

        run_cd(&args, &mut shell);

        assert_eq!(shell.working_dir, Path::new("/usr").to_path_buf());
    }

    #[test]
    fn resolves_home_dir_references() {
        let mut shell = Shell::new(">", Path::new("/usr/bin").to_path_buf());
        let args = vec!["~/"];

        run_cd(&args, &mut shell);

        assert_eq!(shell.working_dir, home_dir().unwrap());
    }
}
