use std::path::PathBuf;
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let (socket, cmd_start) = resolve_socket(&args);
    let cmd = args[cmd_start].as_str();
    let rest = &args[cmd_start + 1..];

    match run_command(&socket, cmd, rest) {
        Ok(msg) => println!("{msg}"),
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("Usage: dioxus-debug <command> [args...]");
    eprintln!("       dioxus-debug --socket <path> <command> [args...]");
    eprintln!();
    eprintln!("Commands: ping, dump, click <selector>, input <selector> <value>,");
    eprintln!("          eval <js>, screenshot <file>, script <file>");
}

fn resolve_socket(args: &[String]) -> (PathBuf, usize) {
    if args.get(1).map(String::as_str) == Some("--socket") {
        if args.len() < 4 {
            eprintln!("--socket requires a path and a command");
            process::exit(1);
        }
        return (PathBuf::from(&args[2]), 3);
    }

    let servers = dioxus_debug::client::find_servers();
    match servers.len() {
        0 => {
            eprintln!("No dioxus-debug servers found");
            process::exit(1);
        }
        1 => (servers[0].clone(), 1),
        n => {
            eprintln!("Found {n} servers, use --socket to pick one:");
            for s in &servers {
                eprintln!("  {}", s.display());
            }
            process::exit(1);
        }
    }
}

fn run_command(socket: &PathBuf, cmd: &str, rest: &[String]) -> Result<String, String> {
    let map_err = |e: peercred_ipc::IpcError| e.to_string();
    match cmd {
        "ping" => dioxus_debug::client::ping(socket)
            .map(|()| "Pong".into())
            .map_err(map_err),
        "dump" => dioxus_debug::client::dump(socket).map_err(map_err),
        "click" => {
            let selector = rest.join(" ");
            dioxus_debug::client::click(socket, &selector)
                .map(|()| "Ok".into())
                .map_err(map_err)
        }
        "input" => run_input(socket, rest),
        "eval" => dioxus_debug::client::eval(socket, &rest.join(" ")).map_err(map_err),
        "screenshot" => run_screenshot(socket, rest),
        "script" => run_script(socket, rest),
        _ => Err(format!("Unknown command: {cmd}")),
    }
}

fn run_input(socket: &PathBuf, rest: &[String]) -> Result<String, String> {
    if rest.len() < 2 {
        return Err("input requires <selector> <value>".into());
    }
    dioxus_debug::client::input(socket, &rest[0], &rest[1..].join(" "))
        .map(|()| "Ok".into())
        .map_err(|e| e.to_string())
}

fn run_screenshot(socket: &PathBuf, rest: &[String]) -> Result<String, String> {
    if rest.is_empty() {
        return Err("screenshot requires a file path".into());
    }
    dioxus_debug::client::screenshot_to_file(socket, &rest[0])
        .map(|()| format!("Saved to {}", rest[0]))
        .map_err(|e| e.to_string())
}

fn run_script(socket: &PathBuf, rest: &[String]) -> Result<String, String> {
    if rest.is_empty() {
        return Err("script requires a file path".into());
    }
    let text = std::fs::read_to_string(&rest[0]).map_err(|e| e.to_string())?;
    let commands = dioxus_debug::script::parse_script(&text)?;
    dioxus_debug::script::run_script(socket, &commands)?;
    Ok("Script completed".into())
}
