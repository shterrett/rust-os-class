use std::str::Split;
use std::path::Path;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct CmdLine<'a> {
    pub name: &'a str,
    pub args: Vec<&'a str>,
    pub background: bool,
    pub stdin: Option<&'a Path>,
    pub stdout: Option<&'a Path>
}

impl<'a> CmdLine<'a> {
    fn empty() -> Self {
        CmdLine {
            name: "",
            args: vec![],
            background: false,
            stdin: None,
            stdout: None
        }
    }
    fn add_arg(mut self, arg: &'a str) -> Self {
        self.args.push(arg);
        self
    }
    fn name(self, name: &'a str) -> Self {
        CmdLine {
            name: name,
            args: self.args,
            background: self.background,
            stdin: self.stdin,
            stdout: self.stdout
        }
    }
    fn background(self, background: bool) -> Self {
        CmdLine {
            name: self.name,
            args: self.args,
            background: background,
            stdin: self.stdin,
            stdout: self.stdout
        }
    }
    fn stdin(self, stdin: &'a Path) -> Self {
        CmdLine {
            name: self.name,
            args: self.args,
            background: self.background,
            stdin: Some(stdin),
            stdout: self.stdout
        }
    }
    fn stdout(self, stdout: &'a Path) -> Self {
        CmdLine {
            name: self.name,
            args: self.args,
            background: self.background,
            stdin: self.stdin,
            stdout: Some(stdout)
        }
    }

    pub fn parse(line: &'a str) -> Result<Self, String> {
        let words = line
            .trim()
            .split(" ");

        parse_line(words, CmdLine::empty())
    }

}

fn parse_line<'a>(mut words: Split<'a, &str>, cmd: CmdLine<'a>) -> Result<CmdLine<'a>, String> {
    let next_word = words.next();
    match next_word {
        Some(">") => {
            words.next()
                .ok_or("must provide target for stdout".to_string())
                .map(|s| Path::new(s))
                .and_then(|stdout| parse_line(words, cmd.stdout(stdout)))
        },
        Some("<") => {
            words.next()
                .ok_or("must provide target for stdin".to_string())
                .and_then(|s| resolve_stdin(s))
                .and_then(|stdin| parse_line(words, cmd.stdin(stdin)))
        },
        Some("&") => {
            parse_line(words, cmd.background(true))
        },
        Some(arg) => {
            if cmd.name == "" {
                parse_line(words, cmd.name(arg))
            } else {
                parse_line(words, cmd.add_arg(arg))
            }
        },
        None => {
            Ok(cmd)
        }
    }
}

fn resolve_stdin(input: &str) -> Result<&Path, String> {
    let path = Path::new(input);
    if  path.is_file() {
        Ok(&path)
    } else {
        Err(format!("{} is not a valid file", input))
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;
    use std::fs::{ File, remove_file };
    use super::CmdLine;

    #[test]
    fn splits_a_string_into_command_and_arguments() {
        let command = "ls -l";

        match CmdLine::parse(command) {
            Ok(cmd_line) => {
                assert_eq!(cmd_line.name, "ls");
                assert_eq!(cmd_line.args, vec!["-l"]);
                assert_eq!(cmd_line.background, false);
            },
            Err(e) => {
                assert!(false, "parse failed with {}", e);
            }
        }
    }

    #[test]
    fn marks_cmd_line_as_background_task_when_ends_in_ampersand() {
        let command = "ls -l &";

        match CmdLine::parse(command) {
            Ok(cmd_line) => {
                assert_eq!(cmd_line.name, "ls");
                assert_eq!(cmd_line.args, vec!["-l"]);
                assert!(cmd_line.background);
            },
            Err(e) => {
                assert!(false, "parse failed with {}", e);
            }
        }
    }

    #[test]
    fn adds_stdout_redirect_when_present() {
        let command = "cat > temp.txt";

        match CmdLine::parse(command) {
            Ok(cmd_line) => {
                assert_eq!(cmd_line.name, "cat");
                assert!(cmd_line.args.is_empty());
                assert_eq!(cmd_line.stdout, Some(Path::new("temp.txt")));
                assert_eq!(cmd_line.stdin, None);
            },
            Err(e) => {
                assert!(false, "parse failed with {}", e);
            }
        }
    }

    #[test]
    fn adds_stdin_redirect_when_present() {
        let _ = File::create("temp.txt").unwrap();
        let command = "cat < temp.txt";

        match CmdLine::parse(command) {
            Ok(cmd_line) => {
                assert_eq!(cmd_line.name, "cat");
                assert!(cmd_line.args.is_empty());
                assert_eq!(cmd_line.stdout, None);
                assert_eq!(cmd_line.stdin, Some(Path::new("temp.txt")));
            },
            Err(e) => {
                assert!(false, "parse failed with {}", e);
            }
        }

        let _ = remove_file(Path::new("temp.txt"));
    }

    #[test]
    fn parsing_fails_when_stdin_not_present() {
        let command = "cat < temp.txt";

        match CmdLine::parse(command) {
            Ok(_) => {
                assert!(false, "parse should fail without temp.txt");
            },
            Err(e) => {
                assert_eq!(e, "temp.txt is not a valid file");
            }
        }
    }
}
