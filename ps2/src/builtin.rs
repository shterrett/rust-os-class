use std::collections::HashSet;
use std::process::exit;
use std::path::{ Path };
use std::env::home_dir;
use std::io::{ self, Write };
use std::fs::File;

use shell::Shell;
use cmd_line::CmdLine;

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

pub fn run<'a>(cmd: &'a CmdLine, shell: &mut Shell) -> Result<(), String> {
    match cmd.name {
        "exit" => exit(0),
        "cd" => run_cd(&cmd, shell),
        "pwd" => run_pwd(&cmd, shell),
        "history" => run_history(&cmd, shell),
        _ => {
            Err(format!("Command not found {}", cmd.name))
        }
    }
}

fn run_cd<'a>(cmd: &'a CmdLine, shell: &mut Shell) -> Result<(), String> {
    let args = &cmd.args;
    let new_path =
        args.first()
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

    match new_path {
        Some(path) => {
            shell.working_dir = path;
            Ok(())
        },
        None => Err(format!("Path not found"))
    }
}

fn run_pwd<'a>(cmd: &'a CmdLine, shell: &Shell) -> Result<(), String>{
    shell.working_dir
        .to_str()
        .map(|dir| output(format!("{}", dir), cmd.stdout))
        .ok_or("Error resolving current directory".to_string())
}

fn run_history<'a>(cmd: &'a CmdLine, shell: &Shell) -> Result<(), String> {
    for cmd_line in shell.history.list(10) {
        output(format!("{}", cmd_line), cmd.stdout);
    }
    Ok(())
}

fn output(text: String, stdout: Option<&Path>) {
    match stdout {
        None =>  {
            let _ = io::stdout().write(text.as_bytes());
        },
        Some(path) => {
             let _ = File::create(path)
                  .and_then(|mut f| f.write_all(text.as_bytes()));
        }
    }
}

#[cfg(test)]
mod test {
    use super::{ run_cd, output };
    use shell::Shell;
    use cmd_line::CmdLine;
    use std::path::Path;
    use std::env::home_dir;
    use std::fs::{ File, remove_file };
    use std::io::Read;

    #[test]
    fn changes_working_directory_to_relative_path() {
        let mut shell = Shell::new(">", Path::new("/").to_path_buf());
        let cmd = CmdLine::parse("cd usr").unwrap();

        let result = run_cd(&cmd, &mut shell);

        assert_eq!(Ok(()), result);
        assert_eq!(shell.working_dir, Path::new("/usr").to_path_buf());
    }

    #[test]
    fn changes_working_directory_to_absolute_path() {
        let mut shell = Shell::new(">", Path::new("/usr").to_path_buf());
        let cmd = CmdLine::parse("cd /usr/bin").unwrap();

        let result = run_cd(&cmd, &mut shell);

        assert_eq!(Ok(()), result);
        assert_eq!(shell.working_dir, Path::new("/usr/bin").to_path_buf());
    }

    #[test]
    fn resolves_parent_dir_references() {
        let mut shell = Shell::new(">", Path::new("/usr/bin").to_path_buf());
        let cmd = CmdLine::parse("cd ..").unwrap();

        let result = run_cd(&cmd, &mut shell);

        assert_eq!(Ok(()), result);
        assert_eq!(shell.working_dir, Path::new("/usr").to_path_buf());
    }

    #[test]
    fn resolves_home_dir_references() {
        let mut shell = Shell::new(">", Path::new("/usr/bin").to_path_buf());
        let cmd = CmdLine::parse("cd ~/").unwrap();

        let result = run_cd(&cmd, &mut shell);

        assert_eq!(Ok(()), result);
        assert_eq!(shell.working_dir, home_dir().unwrap());
    }

    #[test] fn redirects_std_out() {
        let path = Path::new("test/redirect_std_out.txt");

        output("Command Output".to_string(), Some(&path));

        let mut output = "".to_string();
        let _ = File::open(path)
             .unwrap()
             .read_to_string(&mut output);
        assert_eq!(output, "Command Output".to_string());

        let _ = remove_file(path);
    }
}
