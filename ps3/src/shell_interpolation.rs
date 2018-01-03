use std::path::Path;
use std::process::Child;
use std::error::Error;
use regex::{ Regex, Captures };
use cmd_line::{ parse_command, ParsedCommand };
use external::{ run, run_chain };

lazy_static! {
    static ref SHELL_REGEX: Regex = Regex::new(r#"<!--\s*#exec\s+(.+)-->"#).unwrap();
}

pub fn insert_shell_commands(path: &Path, bytes: Vec<u8>) -> Result<Vec<u8>, String> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("shtml") => substitute_shell_command(bytes),
        _ => Ok(bytes)
    }
}

fn substitute_shell_command(bytes: Vec<u8>) -> Result<Vec<u8>, String> {
    String::from_utf8(bytes)
        .map(|s| SHELL_REGEX.replace(&s,
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
            }).into_owned()
        ).map(|replaced| replaced.to_string().into_bytes())
         .map_err(|e| e.description().to_string())
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
    use super::insert_shell_commands;

    #[test]
    fn pass_through_if_not_shtml() {
        let bytes = "<h1><!-- #exec echo \"Hello World\" --></h1>".to_string().into_bytes();
        let expected = Ok(bytes.clone());
        let path = Path::new("world.html");

        assert_eq!(insert_shell_commands(&path, bytes), expected);
    }

    #[test]
    fn executes_shell_command_in_interpolated_shtml_file() {
        let bytes = "<h1><!-- #exec echo \"Hello World\" --></h1>".to_string().into_bytes();
        let expected = Ok("<h1>\"Hello World\"\n</h1>".to_string().into_bytes());
        let path = Path::new("world.shtml");

        let actual = insert_shell_commands(&path, bytes);
        println!("expected {:?}; actual: {:?}", String::from_utf8(expected.clone().unwrap()), String::from_utf8(actual.clone().unwrap()));
        assert_eq!(actual, expected);
    }
}
