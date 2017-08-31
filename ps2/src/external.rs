use std::io::{ self, Write };
use std::process::Command;
use shell::Shell;
use cmd_line::CmdLine;

pub fn run(cmd: CmdLine, shell: &Shell) {
    if cmd.background {
        run_bg(shell, cmd.name, cmd.args);
    } else {
        run_fg(shell, cmd.name, cmd.args);
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
