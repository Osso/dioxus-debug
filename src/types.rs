use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Dump,
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
    Screenshot(String),
}
