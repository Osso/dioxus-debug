use peercred_ipc::{Client, IpcError};
use std::path::{Path, PathBuf};

use crate::types::{Request, Response};

/// Get the raw JSON DOM tree from the app.
pub fn tree_dump_json<P: AsRef<Path>>(socket: P) -> Result<String, IpcError> {
    let resp: Response = Client::call(socket, &Request::TreeDump)?;
    match resp {
        Response::Dom(s) => Ok(s),
        Response::Error(e) => Err(io_err(e)),
        _ => Err(io_err("Unexpected response")),
    }
}

/// Get a formatted indented tree from the app.
pub fn tree_dump<P: AsRef<Path>>(socket: P) -> Result<String, IpcError> {
    let json = tree_dump_json(socket)?;
    crate::tree::format_tree(&json).map_err(io_err)
}

pub fn click<P: AsRef<Path>>(socket: P, selector: &str) -> Result<(), IpcError> {
    let resp: Response = Client::call(
        socket,
        &Request::Click {
            selector: selector.to_string(),
        },
    )?;
    match resp {
        Response::Ok => Ok(()),
        Response::Error(e) => Err(io_err(e)),
        _ => Err(io_err("Unexpected response")),
    }
}

pub fn input<P: AsRef<Path>>(socket: P, selector: &str, value: &str) -> Result<(), IpcError> {
    let resp: Response = Client::call(
        socket,
        &Request::Input {
            selector: selector.to_string(),
            value: value.to_string(),
        },
    )?;
    match resp {
        Response::Ok => Ok(()),
        Response::Error(e) => Err(io_err(e)),
        _ => Err(io_err("Unexpected response")),
    }
}

pub fn eval<P: AsRef<Path>>(socket: P, js: &str) -> Result<String, IpcError> {
    let resp: Response = Client::call(socket, &Request::Eval { js: js.to_string() })?;
    match resp {
        Response::EvalResult(s) => Ok(s),
        Response::Error(e) => Err(io_err(e)),
        _ => Err(io_err("Unexpected response")),
    }
}

/// Get a screenshot as base64-encoded webp.
pub fn screenshot<P: AsRef<Path>>(socket: P) -> Result<String, IpcError> {
    let resp: Response = Client::call(socket, &Request::Screenshot)?;
    match resp {
        Response::Screenshot(s) => Ok(s),
        Response::Error(e) => Err(io_err(e)),
        _ => Err(io_err("Unexpected response")),
    }
}

/// Take a screenshot and save as webp file.
pub fn screenshot_to_file<P: AsRef<Path>, Q: AsRef<Path>>(
    socket: P,
    path: Q,
) -> Result<(), IpcError> {
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD;

    let b64 = screenshot(socket)?;
    let bytes = STANDARD
        .decode(&b64)
        .map_err(|e| io_err(format!("base64 decode: {e}")))?;
    std::fs::write(path, bytes).map_err(IpcError::Io)?;
    Ok(())
}

pub fn ping<P: AsRef<Path>>(socket: P) -> Result<(), IpcError> {
    let resp: Response = Client::call(socket, &Request::Ping)?;
    match resp {
        Response::Pong => Ok(()),
        _ => Err(io_err("Unexpected response")),
    }
}

pub fn find_servers() -> Vec<PathBuf> {
    glob::glob("/tmp/dioxus-debug-*.sock")
        .map(|paths| paths.filter_map(Result::ok).collect())
        .unwrap_or_default()
}

fn io_err(msg: impl Into<String>) -> IpcError {
    IpcError::Io(std::io::Error::other(msg.into()))
}
