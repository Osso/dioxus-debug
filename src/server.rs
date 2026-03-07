pub use crate::types::{Request, Response};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub enum Command {
    Dump {
        respond: oneshot::Sender<String>,
    },
    Click {
        selector: String,
        respond: oneshot::Sender<Result<(), String>>,
    },
    Input {
        selector: String,
        value: String,
        respond: oneshot::Sender<Result<(), String>>,
    },
    Eval {
        js: String,
        respond: oneshot::Sender<Result<String, String>>,
    },
    Screenshot {
        respond: oneshot::Sender<Result<String, String>>,
    },
}

pub type CommandReceiver = mpsc::Receiver<Command>;

pub fn socket_path() -> PathBuf {
    PathBuf::from(format!("/tmp/dioxus-debug-{}.sock", std::process::id()))
}

pub struct SocketGuard {
    path: PathBuf,
    shutdown: Arc<AtomicBool>,
}

impl Drop for SocketGuard {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        let _ = std::fs::remove_file(&self.path);
        eprintln!("[dioxus-debug] Cleaned up {}", self.path.display());
    }
}

fn cleanup_stale_sockets() {
    let Ok(entries) = glob::glob("/tmp/dioxus-debug-*.sock") else {
        return;
    };
    for entry in entries.flatten() {
        cleanup_socket_if_stale(&entry);
    }
}

fn cleanup_socket_if_stale(entry: &std::path::Path) {
    let Some(filename) = entry.file_name().and_then(|f| f.to_str()) else {
        return;
    };
    let Some(pid_str) = filename
        .strip_prefix("dioxus-debug-")
        .and_then(|s| s.strip_suffix(".sock"))
    else {
        return;
    };
    let Ok(pid) = pid_str.parse::<i32>() else {
        return;
    };
    let alive = unsafe { libc::kill(pid, 0) } == 0;
    if !alive && std::fs::remove_file(entry).is_ok() {
        eprintln!(
            "[dioxus-debug] Cleaned up stale socket: {}",
            entry.display()
        );
    }
}

pub fn init() -> (mpsc::Receiver<Command>, SocketGuard) {
    cleanup_stale_sockets();

    let (cmd_tx, cmd_rx) = mpsc::channel::<Command>(16);
    let path = socket_path();
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = shutdown.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(run_server(cmd_tx, shutdown_clone));
    });

    let guard = SocketGuard { path, shutdown };
    (cmd_rx, guard)
}

async fn run_server(cmd_tx: mpsc::Sender<Command>, shutdown: Arc<AtomicBool>) {
    use peercred_ipc::Server;

    let path = socket_path();
    let server = match Server::bind(&path) {
        Ok(s) => {
            eprintln!("[dioxus-debug] Listening on {}", path.display());
            s
        }
        Err(e) => {
            eprintln!("[dioxus-debug] Failed to bind: {}", e);
            return;
        }
    };

    loop {
        if shutdown.load(Ordering::SeqCst) {
            break;
        }

        let mut conn = match accept_connection(&server).await {
            Some(c) => c,
            None => continue,
        };

        let request: Result<Request, _> = conn.read().await;
        match request {
            Ok(req) => dispatch_request(req, &mut conn, &cmd_tx).await,
            Err(e) => eprintln!("[dioxus-debug] Read error: {}", e),
        }
    }
}

async fn accept_connection(server: &peercred_ipc::Server) -> Option<peercred_ipc::Connection> {
    match tokio::time::timeout(std::time::Duration::from_millis(100), server.accept()).await {
        Ok(Ok((conn, _caller))) => Some(conn),
        Ok(Err(e)) => {
            eprintln!("[dioxus-debug] Accept error: {}", e);
            None
        }
        Err(_) => None,
    }
}

async fn dispatch_request(
    req: Request,
    conn: &mut peercred_ipc::Connection,
    cmd_tx: &mpsc::Sender<Command>,
) {
    match req {
        Request::Dump => {
            let cmd = |tx| Command::Dump { respond: tx };
            handle_string_command(conn, cmd_tx, cmd).await;
        }
        Request::Click { selector } => {
            let cmd = |tx| Command::Click {
                selector,
                respond: tx,
            };
            send_result_command(conn, cmd_tx, cmd, 2).await;
        }
        Request::Input { selector, value } => {
            let cmd = |tx| Command::Input {
                selector,
                value,
                respond: tx,
            };
            send_result_command(conn, cmd_tx, cmd, 2).await;
        }
        Request::Eval { js } => {
            let cmd = |tx| Command::Eval { js, respond: tx };
            handle_eval_command(conn, cmd_tx, cmd).await;
        }
        Request::Ping => {
            let _ = conn.write(&Response::Pong).await;
        }
        Request::Screenshot => {
            let cmd = |tx| Command::Screenshot { respond: tx };
            handle_eval_command(conn, cmd_tx, cmd).await;
        }
    }
}

async fn handle_string_command<F>(
    conn: &mut peercred_ipc::Connection,
    cmd_tx: &mpsc::Sender<Command>,
    make_cmd: F,
) where
    F: FnOnce(oneshot::Sender<String>) -> Command,
{
    let (tx, rx) = oneshot::channel();
    if cmd_tx.send(make_cmd(tx)).await.is_err() {
        let _ = conn.write(&Response::Error("App closed".into())).await;
        return;
    }
    match tokio::time::timeout(std::time::Duration::from_secs(5), rx).await {
        Ok(Ok(s)) => {
            let _ = conn.write(&Response::Dom(s)).await;
        }
        _ => {
            let _ = conn.write(&Response::Error("Timeout".into())).await;
        }
    }
}

async fn send_result_command<F>(
    conn: &mut peercred_ipc::Connection,
    cmd_tx: &mpsc::Sender<Command>,
    make_cmd: F,
    timeout_secs: u64,
) where
    F: FnOnce(oneshot::Sender<Result<(), String>>) -> Command,
{
    let (tx, rx) = oneshot::channel();
    if cmd_tx.send(make_cmd(tx)).await.is_err() {
        let _ = conn.write(&Response::Error("App closed".into())).await;
        return;
    }
    match tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), rx).await {
        Ok(Ok(Ok(()))) => {
            let _ = conn.write(&Response::Ok).await;
        }
        Ok(Ok(Err(e))) => {
            let _ = conn.write(&Response::Error(e)).await;
        }
        _ => {
            let _ = conn.write(&Response::Error("Timeout".into())).await;
        }
    }
}

async fn handle_eval_command<F>(
    conn: &mut peercred_ipc::Connection,
    cmd_tx: &mpsc::Sender<Command>,
    make_cmd: F,
) where
    F: FnOnce(oneshot::Sender<Result<String, String>>) -> Command,
{
    let (tx, rx) = oneshot::channel();
    if cmd_tx.send(make_cmd(tx)).await.is_err() {
        let _ = conn.write(&Response::Error("App closed".into())).await;
        return;
    }
    match tokio::time::timeout(std::time::Duration::from_secs(5), rx).await {
        Ok(Ok(Ok(s))) => {
            let _ = conn.write(&Response::EvalResult(s)).await;
        }
        Ok(Ok(Err(e))) => {
            let _ = conn.write(&Response::Error(e)).await;
        }
        _ => {
            let _ = conn.write(&Response::Error("Timeout".into())).await;
        }
    }
}
