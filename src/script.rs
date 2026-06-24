#[cfg(feature = "script")]
use std::path::Path;
#[cfg(feature = "script")]
use std::time::Duration;

#[cfg(feature = "script")]
use crate::client;

#[derive(Debug, PartialEq)]
pub enum ScriptCommand {
    Click(String),
    Input { selector: String, value: String },
    Wait(u64),
    Screenshot(String),
    TreeDump,
    Eval(String),
    Ping,
}

pub fn parse_script(text: &str) -> Result<Vec<ScriptCommand>, String> {
    let mut commands = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let cmd = parse_line(line).map_err(|e| format!("Line {}: {}", i + 1, e))?;
        commands.push(cmd);
    }
    Ok(commands)
}

fn parse_line(line: &str) -> Result<ScriptCommand, String> {
    let (cmd, rest) = line.split_once(' ').unwrap_or((line, ""));
    let rest = rest.trim();
    match cmd {
        "click" => {
            if rest.is_empty() {
                return Err("click requires a selector".into());
            }
            Ok(ScriptCommand::Click(rest.to_string()))
        }
        "input" => {
            let (selector, value) = rest
                .split_once(' ')
                .ok_or("input requires selector and value")?;
            Ok(ScriptCommand::Input {
                selector: selector.to_string(),
                value: value.to_string(),
            })
        }
        "wait" => {
            let ms: u64 = rest.parse().map_err(|_| "wait requires milliseconds")?;
            Ok(ScriptCommand::Wait(ms))
        }
        "screenshot" => {
            if rest.is_empty() {
                return Err("screenshot requires a file path".into());
            }
            Ok(ScriptCommand::Screenshot(rest.to_string()))
        }
        "tree-dump" => Ok(ScriptCommand::TreeDump),
        "eval" => {
            if rest.is_empty() {
                return Err("eval requires a JS expression".into());
            }
            Ok(ScriptCommand::Eval(rest.to_string()))
        }
        "ping" => Ok(ScriptCommand::Ping),
        _ => Err(format!("Unknown command: {cmd}")),
    }
}

#[cfg(feature = "script")]
pub fn run_script<P: AsRef<Path>>(socket: P, commands: &[ScriptCommand]) -> Result<(), String> {
    let socket = socket.as_ref();
    for cmd in commands {
        run_command(socket, cmd)?;
    }
    Ok(())
}

#[cfg(feature = "script")]
fn run_command(socket: &Path, cmd: &ScriptCommand) -> Result<(), String> {
    match cmd {
        ScriptCommand::Click(selector) => {
            client::click(socket, selector).map_err(|e| format!("click: {e}"))?;
        }
        ScriptCommand::Input { selector, value } => {
            client::input(socket, selector, value).map_err(|e| format!("input: {e}"))?;
        }
        ScriptCommand::Wait(ms) => {
            std::thread::sleep(Duration::from_millis(*ms));
        }
        ScriptCommand::Screenshot(path) => {
            client::screenshot_to_file(socket, path).map_err(|e| format!("screenshot: {e}"))?;
        }
        ScriptCommand::TreeDump => {
            let tree = client::tree_dump(socket).map_err(|e| format!("tree-dump: {e}"))?;
            println!("{tree}");
        }
        ScriptCommand::Eval(js) => {
            let result = client::eval(socket, js).map_err(|e| format!("eval: {e}"))?;
            println!("{result}");
        }
        ScriptCommand::Ping => {
            client::ping(socket).map_err(|e| format!("ping: {e}"))?;
            println!("Pong");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ScriptCommand, parse_script};

    #[test]
    fn parses_commands_and_skips_blank_lines_and_comments() {
        let script = r#"
            # setup
            ping
            click #save
            input #name Alessio Deiana
            wait 250
            screenshot /tmp/app.webp
            tree-dump
            eval return document.title
        "#;

        let commands = parse_script(script).unwrap();

        assert_eq!(
            commands,
            vec![
                ScriptCommand::Ping,
                ScriptCommand::Click("#save".to_string()),
                ScriptCommand::Input {
                    selector: "#name".to_string(),
                    value: "Alessio Deiana".to_string()
                },
                ScriptCommand::Wait(250),
                ScriptCommand::Screenshot("/tmp/app.webp".to_string()),
                ScriptCommand::TreeDump,
                ScriptCommand::Eval("return document.title".to_string()),
            ]
        );
    }

    #[test]
    fn annotates_parse_errors_with_line_number() {
        let err = parse_script("\nwait soon").unwrap_err();

        assert_eq!(err, "Line 2: wait requires milliseconds");
    }

    #[test]
    fn rejects_missing_command_arguments() {
        assert_eq!(
            parse_script("click").unwrap_err(),
            "Line 1: click requires a selector"
        );
        assert_eq!(
            parse_script("input #name").unwrap_err(),
            "Line 1: input requires selector and value"
        );
        assert_eq!(
            parse_script("screenshot").unwrap_err(),
            "Line 1: screenshot requires a file path"
        );
        assert_eq!(
            parse_script("eval").unwrap_err(),
            "Line 1: eval requires a JS expression"
        );
        assert_eq!(
            parse_script("hover #name").unwrap_err(),
            "Line 1: Unknown command: hover"
        );
    }
}
