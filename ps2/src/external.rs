use std::io::{ self, Write };
use std::process::{ Command, Stdio };
use shell::Shell;
use cmd_line::CmdLine;
use std::fs::{ File, OpenOptions };
use std::path::Path;
use std::error::Error;

pub fn run(cmd: &CmdLine, shell: &Shell) -> Result<(), String> {
    if cmd.background {
        run_bg(shell, cmd)
    } else {
        run_fg(shell, cmd)
    }
}

fn run_fg(shell: &Shell, cmd: &CmdLine) -> Result<(), String> {
    let stdin = use_std_io(cmd.stdin);
    Command::new(cmd.name)
        .args(&cmd.args)
        .current_dir(shell.working_dir.clone())
        .stdin(stdin)
        .output()
        .map(|output| output.stdout)
        .and_then(|output| write_output(cmd.stdout, output.as_slice()))
        .map(|_| ())
        .map_err(|e| e.description().to_string())
}

fn run_bg(shell: &Shell, cmd: &CmdLine) -> Result<(), String> {
    let stdin = use_std_io(cmd.stdin);
    let stdout = use_std_io(cmd.stdout);
    Command::new(cmd.name)
        .args(&cmd.args)
        .current_dir(shell.working_dir.clone())
        .stdin(stdin)
        .stdout(stdout)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.description().to_string())
}

fn write_output(out_path: Option<&Path>, output: &[u8]) -> io::Result<usize> {
    match out_path {
        None => {
            io::stdout()
                .write(&output)
        },
        Some(path) => {
            File::create(path)
                .and_then(|mut f| f.write(&output))
        }
    }
}

fn use_std_io(io_path: Option<&Path>) -> Stdio {
    match io_path {
        Some(path) => {
            let mut options = OpenOptions::new();
            let file = options.read(true).write(true).truncate(true).open(path).unwrap();
            Stdio::from(file)
        },
        None => Stdio::inherit()
    }
}
