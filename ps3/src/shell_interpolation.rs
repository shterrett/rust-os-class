use std::path::Path;
use std::process::Child;
use std::error::Error;
use std::fs::File;
use std::io::{ Read, BufReader };
use regex::{ Regex, Captures };
use cmd_line::{ parse_command, ParsedCommand };
use external::{ run, run_chain };
use http::Payload;

lazy_static! {
    static ref SHELL_REGEX: Regex = Regex::new(r#"<!--\s*#exec\s+(.+)-->"#).unwrap();
}

pub fn insert_shell_commands(path: &Path, payload: Payload) -> Result<Payload, String> {
    match payload {
        Payload::Stream(file) => insert_shell_commands_file(path, file),
        Payload::Block(string) => substitute_shell_command(string)
    }
}

fn insert_shell_commands_file(path: &Path, mut file: BufReader<File>) -> Result<Payload, String> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("shtml") => {
            let mut contents = String::new();
            match file.read_to_string(&mut contents) {
                Ok(_) => substitute_shell_command(contents),
                Err(e) => Err(e.description().to_string())
            }
        }
        _ => Ok(Payload::Stream(file))
    }
}


fn substitute_shell_command(contents: String) -> Result<Payload, String> {
    let replaced_string = SHELL_REGEX.replace(&contents,
        |captured: &Captures| {
            match parse_command(&captured[1]) {
                Ok(ParsedCommand::SingleCommand(cmd)) => {
                    output(run(&cmd))
                }
                Ok(ParsedCommand::PipeChain(cmds)) => {
                    output(run_chain(&cmds))
                }
                Err(e) => e
            }
        }).into_owned().to_string();
    Ok(Payload::Block(replaced_string))
}

fn output(cmd: Result<Child, String>) -> String {
    cmd.and_then(|c| c.wait_with_output().map_err(|e| e.description().to_string()))
       .map(|c| c.stdout)
       .and_then(|bytes| {
           String::from_utf8(bytes)
               .map_err(|e| e.description().to_string())
       })
       .unwrap_or_else(|e| e)
}

#[cfg(test)]
mod test {
    use std::path::Path;
    use std::fs::File;
    use std::io::{ BufReader, Read };
    use http::Payload;
    use super::insert_shell_commands;

    #[test]
    fn pass_through_if_not_shtml() {
        let path = Path::new("test/improper_template.html");
        let mut expected = String::new();
        let mut expect_file = File::open(&path).unwrap();
        let _ = expect_file.read_to_string(&mut expected);

        let test_file = Payload::Stream(BufReader::new(File::open(&path).unwrap()));
        let interpolated = insert_shell_commands(&path, test_file).unwrap();

        match interpolated {
            Payload::Stream(mut bfr) => {
                let mut actual = String::new();
                let _  = bfr.read_to_string(&mut actual).unwrap();
                assert_eq!(actual, expected);
            }
            Payload::Block(_) => assert!(false, "Transformed file")
        }
    }

    #[test]
    fn executes_shell_command_in_interpolated_shtml_file() {
        let path = Path::new("test/world.shtml");
        let expected = "<h1>\"Hello World\"\n</h1>\n".to_string();

        let test_file = Payload::Stream(BufReader::new(File::open(&path).unwrap()));
        let interpolated = insert_shell_commands(&path, test_file).unwrap();

        match interpolated {
            Payload::Stream(_) =>  assert!(false, "Did not transform file"),
            Payload::Block(interpolated) => {
                assert_eq!(interpolated, expected);
            }
        }
    }
}
