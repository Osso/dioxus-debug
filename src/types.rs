use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    TreeDump,
    Click { selector: String },
    Input { selector: String, value: String },
    Eval { js: String },
    Screenshot,
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Dom(String),
    Ok,
    Pong,
    Error(String),
    EvalResult(String),
    /// Base64-encoded webp image data
    Screenshot(String),
}
