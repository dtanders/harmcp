use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Entry {
    #[serde(rename = "startedDateTime")]
    pub started_date_time: String,
    pub time: f64,
    pub request: Request,
    pub response: Response,
    pub timings: Timings,
    #[serde(rename = "_initiator")]
    pub initiator: Option<Initiator>,
}

#[derive(Debug, Deserialize)]
pub struct Request {
    pub method: String,
    pub url: String,
    pub headers: Vec<Header>,
    #[serde(rename = "postData")]
    pub post_data: Option<PostData>,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub status: u16,
    #[serde(rename = "statusText")]
    pub status_text: String,
    pub headers: Vec<Header>,
    pub content: Content,
}

#[derive(Debug, Deserialize)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct PostData {
    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Content {
    pub size: i64,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Timings {
    #[serde(default)]
    pub blocked: Option<f64>,
    #[serde(default)]
    pub dns: Option<f64>,
    #[serde(default)]
    pub connect: Option<f64>,
    pub send: f64,
    pub wait: f64,
    pub receive: f64,
}

#[derive(Debug, Deserialize)]
pub struct Initiator {
    #[serde(rename = "type")]
    pub initiator_type: String,
    pub stack: Option<Stack>,
}

#[derive(Debug, Deserialize)]
pub struct Stack {
    #[serde(rename = "callFrames")]
    pub call_frames: Vec<CallFrame>,
}

#[derive(Debug, Deserialize)]
pub struct CallFrame {
    #[serde(rename = "functionName", default)]
    pub function_name: String,
    pub url: String,
    #[serde(rename = "lineNumber")]
    pub line_number: u32,
    #[serde(rename = "columnNumber", default)]
    pub column_number: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_ENTRY: &str = r#"{
        "startedDateTime": "2024-01-01T00:00:00.000Z",
        "time": 123.4,
        "request": {
            "method": "GET",
            "url": "https://example.com/api",
            "headers": [{"name": "Accept", "value": "application/json"}],
            "headersSize": -1,
            "bodySize": -1
        },
        "response": {
            "status": 200,
            "statusText": "OK",
            "headers": [{"name": "Content-Type", "value": "application/json"}],
            "content": {"size": 512, "mimeType": "application/json"},
            "redirectURL": "",
            "headersSize": -1,
            "bodySize": 512
        },
        "timings": {"send": 0.5, "wait": 100.0, "receive": 16.9}
    }"#;

    const ENTRY_WITH_INITIATOR: &str = r#"{
        "startedDateTime": "2024-01-01T00:00:01.000Z",
        "time": 50.0,
        "request": {
            "method": "POST",
            "url": "https://example.com/api/login",
            "headers": [],
            "postData": {"mimeType": "application/json", "text": "{\"user\":\"alice\"}"},
            "headersSize": -1,
            "bodySize": 17
        },
        "response": {
            "status": 401,
            "statusText": "Unauthorized",
            "headers": [],
            "content": {"size": 0, "mimeType": "application/json"},
            "redirectURL": "",
            "headersSize": -1,
            "bodySize": 0
        },
        "timings": {"send": 1.0, "wait": 48.0, "receive": 1.0},
        "_initiator": {
            "type": "script",
            "stack": {
                "callFrames": [
                    {"functionName": "login", "url": "https://example.com/app.js", "lineNumber": 10, "columnNumber": 5}
                ]
            }
        }
    }"#;

    #[test]
    fn deserialize_minimal_entry() {
        let entry: Entry = serde_json::from_str(MINIMAL_ENTRY).unwrap();
        assert_eq!(entry.request.method, "GET");
        assert_eq!(entry.response.status, 200);
        assert_eq!(entry.response.content.size, 512);
        assert!(entry.initiator.is_none());
    }

    #[test]
    fn deserialize_entry_with_initiator() {
        let entry: Entry = serde_json::from_str(ENTRY_WITH_INITIATOR).unwrap();
        assert_eq!(entry.response.status, 401);
        let init = entry.initiator.unwrap();
        assert_eq!(init.initiator_type, "script");
        let frame = &init.stack.unwrap().call_frames[0];
        assert_eq!(frame.function_name, "login");
        assert_eq!(frame.line_number, 10);
    }

    #[test]
    fn deserialize_post_data() {
        let entry: Entry = serde_json::from_str(ENTRY_WITH_INITIATOR).unwrap();
        let body = entry.request.post_data.unwrap();
        assert_eq!(body.text.unwrap(), r#"{"user":"alice"}"#);
    }
}
