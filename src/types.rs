use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Request {
    TreeDump,
    Click { selector: String },
    Input { selector: String, value: String },
    Eval { js: String },
    Screenshot,
    Ping,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Response {
    Dom(String),
    Ok,
    Pong,
    Error(String),
    EvalResult(String),
    /// Base64-encoded webp image data
    Screenshot(String),
}

#[cfg(test)]
mod tests {
    use super::{Request, Response};

    #[test]
    fn request_round_trips_click_payload() {
        let request = Request::Click {
            selector: "#submit".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let decoded: Request = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded, request);
    }

    #[test]
    fn response_round_trips_error_payload() {
        let response = Response::Error("Element not found".to_string());

        let json = serde_json::to_string(&response).unwrap();
        let decoded: Response = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded, response);
    }
}
