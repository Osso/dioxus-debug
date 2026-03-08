#[cfg(any(feature = "server", feature = "client"))]
pub mod types;

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "server")]
mod js;

#[cfg(feature = "server")]
pub mod screenshot;

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "script")]
pub mod script;

#[cfg(feature = "client")]
pub mod tree;

#[cfg(feature = "server")]
pub use server::{Command, CommandReceiver};

/// Hook that captures a screenshot after the first render, saves it, and exits.
///
/// ```ignore
/// dioxus_debug::use_screenshot("output.webp");
/// ```
#[cfg(feature = "server")]
pub fn use_screenshot(path: &str) {
    use dioxus::prelude::*;

    let path = path.to_string();
    let mut done = use_signal(|| false);

    if !*done.read() {
        done.set(true);
        spawn(async move {
            // Wait a bit for the UI to fully render
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            match screenshot::screenshot_to_file(&path).await {
                Ok(()) => eprintln!("Saved screenshot to {path}"),
                Err(e) => eprintln!("Screenshot failed: {e}"),
            }
            std::process::exit(0);
        });
    }
}

/// Dioxus hook that spawns the IPC debug server and bridges commands to `document::eval()`.
///
/// Add to any component (typically the root):
/// ```ignore
/// #[cfg(debug_assertions)]
/// dioxus_debug::use_debug_server();
/// ```
#[cfg(feature = "server")]
pub fn use_debug_server() {
    use dioxus::prelude::*;

    let mut initialized = use_signal(|| false);

    if !*initialized.read() {
        initialized.set(true);
        let (mut rx, guard) = server::init();
        // Leak the guard to keep the socket alive for the app lifetime
        std::mem::forget(guard);
        spawn(async move {
            while let Some(cmd) = rx.recv().await {
                dispatch_command(cmd).await;
            }
        });
    }
}

#[cfg(feature = "server")]
async fn dispatch_command(cmd: server::Command) {
    match cmd {
        server::Command::TreeDump { respond } => {
            let _ = respond.send(eval_to_string(js::DUMP).await);
        }
        server::Command::Click { selector, respond } => {
            let _ = respond.send(eval_to_result(&js::click_js(&selector)).await);
        }
        server::Command::Input {
            selector,
            value,
            respond,
        } => {
            let _ = respond.send(eval_to_result(&js::input_js(&selector, &value)).await);
        }
        server::Command::Eval { js, respond } => {
            let _ = respond.send(eval_to_string_result(&js).await);
        }
        server::Command::Screenshot { respond } => {
            let _ = respond.send(screenshot::capture_screenshot().await);
        }
    }
}

/// Eval JS and return the result as a string (for tree-dump).
#[cfg(feature = "server")]
async fn eval_to_string(js: &str) -> String {
    use dioxus::prelude::*;
    match document::eval(js).await {
        Ok(v) => v.to_string(),
        Err(e) => format!("{{\"error\": \"{e}\"}}"),
    }
}

/// Eval JS and interpret "error:..." prefix as Err.
/// Treats `EvalError::Finished` as success (click triggered a re-render).
#[cfg(feature = "server")]
async fn eval_to_result(js: &str) -> Result<(), String> {
    use dioxus::prelude::*;
    match document::eval(js).await {
        Ok(v) => {
            let s = v.to_string();
            if let Some(msg) = s.trim_matches('"').strip_prefix("error:") {
                Err(msg.to_string())
            } else {
                Ok(())
            }
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("Finished") {
                Ok(())
            } else {
                Err(msg)
            }
        }
    }
}

/// Eval JS and return Ok(string) or Err(string).
#[cfg(feature = "server")]
async fn eval_to_string_result(js: &str) -> Result<String, String> {
    use dioxus::prelude::*;
    match document::eval(js).await {
        Ok(v) => Ok(v.to_string()),
        Err(e) => Err(e.to_string()),
    }
}
