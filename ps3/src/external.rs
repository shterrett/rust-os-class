// Copied with modification from ps2

use std::process::{ Command, Stdio, Child };
use cmd_line::{ CmdLine, CmdIO };
use std::fs::OpenOptions;
use std::error::Error;

pub fn run(cmd: &CmdLine) -> Result<Child, String> {
    let stdin = use_std_io(&cmd.stdin);
    let stdout = use_std_io(&cmd.stdout);
    Command::new(cmd.name)
        .args(&cmd.args)
        .stdin(stdin)
        .stdout(stdout)
        .spawn()
        .map_err(|e| e.description().to_string())
}

pub fn run_chain(cmds: &Vec<CmdLine>) -> Result<Child, String> {
    let cmd = &cmds[0];
    let first = run(cmd);

    run_chain_iter(&cmds[1..], first)
}

fn run_chain_iter(cmds: &[CmdLine], previous: Result<Child, String>) -> Result<Child, String> {
    match previous {
        Ok(prev_cmd) => {
            if cmds.is_empty() {
                return Ok(prev_cmd)
            } else {
                if let Some(stdout) = prev_cmd.stdout {
                    let cmd = &cmds[0];
                    let next = Command::new(cmd.name)
                            .args(&cmd.args)
                            .stdin(stdout)
                            .stdout(use_std_io(&cmd.stdout))
                            .spawn()
                            .map_err(|e| e.description().to_string());

                    run_chain_iter(&cmds[1..], next)
                } else {
                    Err("Could not open stdout".to_string())
                }
            }
        },
        Err(e) => Err(e)
    }
}

fn use_std_io(io_path: &CmdIO) -> Stdio {
    match io_path {
        &CmdIO::File(path) => {
            let mut options = OpenOptions::new();
            let file = options.read(true).write(true).truncate(true).open(path).unwrap();
            Stdio::from(file)
        },
        &CmdIO::Console => Stdio::piped(),
        &CmdIO::Pipe => Stdio::piped()
    }
}
