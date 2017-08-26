pub struct CmdLine<'a> {
    pub name: &'a str,
    pub args: Vec<&'a str>,
    pub background: bool
}

impl<'a> CmdLine<'a> {
    pub fn parse(line: &'a str) -> Option<Self> {
        let background = line.ends_with("&");

        let argv: Vec<&str> = line
            .trim_right_matches("&")
            .split(' ')
            .filter_map(|x| {
            if x == "" {
                None
            } else {
                Some(x)
            }
        }).collect();

        argv.split_first()
            .map(|(&name, args)|
                CmdLine {
                    name: name,
                    args:  args.to_vec(),
                    background: background
                }
            )
    }
}

#[cfg(test)]
mod test {
    use super::CmdLine;

    #[test]
    fn splits_a_string_into_command_and_arguments() {
        let command = "ls -l";

        match CmdLine::parse(command) {
            Some(cmd_line) => {
                assert_eq!(cmd_line.name, "ls");
                assert_eq!(cmd_line.args, vec!["-l"]);
                assert_eq!(cmd_line.background, false);
            },
            None => {
                assert!(false, "parse failed");
            }
        }
    }

    #[test]
    fn marks_cmd_line_as_background_task_when_ends_in_ampersand() {
        let command = "ls -l &";

        match CmdLine::parse(command) {
            Some(cmd_line) => {
                assert_eq!(cmd_line.name, "ls");
                assert_eq!(cmd_line.args, vec!["-l"]);
                assert!(cmd_line.background);
            },
            None => {
                assert!(false, "parse failed");
            }
        }
    }
}
