use std::io::{ self, Write };
use std::process::Command;
use shell::Shell;

pub fn run_external(shell: &Shell, cmd_path: &str, args:  Vec<&str>, background: bool) {
    if background {
        run_bg(shell, cmd_path, args);
    } else {
        run_fg(shell, cmd_path, args);
    }
}

fn run_fg(shell: &Shell, cmd_path: &str, args: Vec<&str>) {
    io::stdout()
        .write(&Command::new(cmd_path)
               .args(args)
               .current_dir(shell.working_dir.clone())
               .output()
               .unwrap()
               .stdout)
        .unwrap();
}

fn run_bg(shell: &Shell, cmd_path: &str, args: Vec<&str>) {
    Command::new(cmd_path)
        .args(args)
        .current_dir(shell.working_dir.clone())
        .spawn()
        .unwrap();
}
