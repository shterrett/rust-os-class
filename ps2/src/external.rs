use std::io::{ self, Write };
use std::process::{ Command, Stdio };
use shell::Shell;
use cmd_line::CmdLine;
use std::fs::{ File, OpenOptions };
use std::path::Path;

pub fn run(cmd: CmdLine, shell: &Shell) {
    if cmd.background {
        run_bg(shell, cmd);
    } else {
        run_fg(shell, cmd);
    }
}

fn run_fg(shell: &Shell, cmd: CmdLine) {
    let stdin = use_std_io(cmd.stdin);
    let output = &Command::new(cmd.name)
        .args(cmd.args)
        .current_dir(shell.working_dir.clone())
        .stdin(stdin)
        .output()
        .unwrap()
        .stdout;
    write_output(cmd.stdout, output);
}

fn run_bg(shell: &Shell, cmd: CmdLine) {
    let stdin = use_std_io(cmd.stdin);
    let stdout = use_std_io(cmd.stdout);
    Command::new(cmd.name)
        .args(cmd.args)
        .current_dir(shell.working_dir.clone())
        .stdin(stdin)
        .stdout(stdout)
        .spawn()
        .unwrap();
}

fn write_output(out_path: Option<&Path>, output: &[u8]) {
    match out_path {
        None => {
            io::stdout()
                .write(&output)
                .unwrap();
        },
        Some(path) => {
            File::create(path)
                .and_then(|mut f| f.write(&output))
                .unwrap();
        }
    }
}

fn use_std_io(io_path: Option<&Path>) -> Stdio {
    match io_path {
        Some(path) => {
            let mut options = OpenOptions::new();
            let mut file = options.read(true).write(true).truncate(true).open(path).unwrap();
            Stdio::from(file)
        },
        None => Stdio::inherit()
    }
}
