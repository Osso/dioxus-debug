use std::path::Path;
use std::time::Duration;

use crate::client;

#[derive(Debug)]
pub enum ScriptCommand {
    Click(String),
    Input { selector: String, value: String },
    Wait(u64),
    Screenshot(String),
    Dump,
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
        "dump" => Ok(ScriptCommand::Dump),
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

pub fn run_script<P: AsRef<Path>>(socket: P, commands: &[ScriptCommand]) -> Result<(), String> {
    let socket = socket.as_ref();
    for cmd in commands {
        run_command(socket, cmd)?;
    }
    Ok(())
}

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
        ScriptCommand::Dump => {
            let dom = client::dump(socket).map_err(|e| format!("dump: {e}"))?;
            println!("{dom}");
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
