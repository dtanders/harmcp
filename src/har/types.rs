use serde::Deserialize;

fn unknown_size() -> i64 {
    -1
}

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
    #[serde(rename = "serverIPAddress", default)]
    pub server_ip_address: Option<String>,
    #[serde(default)]
    pub connection: Option<String>,
    #[serde(default)]
    pub pageref: Option<String>,
    #[serde(rename = "_webSocketMessages", default)]
    pub websocket_messages: Option<Vec<WsMessage>>,
}

#[derive(Debug, Deserialize)]
pub struct Request {
    pub method: String,
    pub url: String,
    pub headers: Vec<Header>,
    #[serde(rename = "queryString", default)]
    pub query_string: Vec<Header>,
    #[serde(default)]
    pub cookies: Vec<Cookie>,
    #[serde(rename = "headersSize", default = "unknown_size")]
    pub headers_size: i64,
    #[serde(rename = "bodySize", default = "unknown_size")]
    pub body_size: i64,
    #[serde(rename = "postData")]
    pub post_data: Option<PostData>,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub status: u16,
    #[serde(rename = "statusText")]
    pub status_text: String,
    pub headers: Vec<Header>,
    #[serde(default)]
    pub cookies: Vec<Cookie>,
    pub content: Content,
    #[serde(rename = "redirectURL", default)]
    pub redirect_url: String,
    #[serde(rename = "headersSize", default = "unknown_size")]
    pub headers_size: i64,
    #[serde(rename = "bodySize", default = "unknown_size")]
    pub body_size: i64,
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
    #[serde(default)]
    pub encoding: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub expires: Option<String>,
    #[serde(rename = "httpOnly", default)]
    pub http_only: Option<bool>,
    #[serde(default)]
    pub secure: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct WsMessage {
    #[serde(rename = "type")]
    pub message_type: String,
    pub time: f64,
    #[serde(default)]
    pub opcode: i64,
    #[serde(default)]
    pub data: String,
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

/// Shared test-entry builder. Tests mutate fields as needed.
#[cfg(test)]
pub fn test_entry() -> Entry {
    Entry {
        started_date_time: "2024-01-01T00:00:00.000Z".to_string(),
        time: 100.0,
        request: Request {
            method: "GET".to_string(),
            url: "https://example.com/api".to_string(),
            headers: vec![],
            query_string: vec![],
            cookies: vec![],
            headers_size: -1,
            body_size: -1,
            post_data: None,
        },
        response: Response {
            status: 200,
            status_text: "OK".to_string(),
            headers: vec![],
            cookies: vec![],
            content: Content {
                size: 512,
                mime_type: "application/json".to_string(),
                text: None,
                encoding: None,
            },
            redirect_url: String::new(),
            headers_size: -1,
            body_size: -1,
        },
        timings: Timings {
            blocked: None,
            dns: None,
            connect: None,
            send: 1.0,
            wait: 90.0,
            receive: 9.0,
        },
        initiator: None,
        server_ip_address: None,
        connection: None,
        pageref: None,
        websocket_messages: None,
    }
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

    const ENTRY_WITH_EXTRAS: &str = r#"{
        "startedDateTime": "2024-01-01T00:00:02.000Z",
        "time": 10.0,
        "request": {
            "method": "GET",
            "url": "https://example.com/img.png?w=64",
            "headers": [],
            "queryString": [{"name": "w", "value": "64"}],
            "cookies": [{"name": "session", "value": "abc123"}],
            "headersSize": 120,
            "bodySize": 0
        },
        "response": {
            "status": 301,
            "statusText": "Moved Permanently",
            "headers": [],
            "cookies": [{"name": "session", "value": "abc123", "path": "/", "httpOnly": true, "secure": true}],
            "content": {"size": 4, "mimeType": "image/png", "text": "iVBORw==", "encoding": "base64"},
            "redirectURL": "https://cdn.example.com/img.png",
            "headersSize": 200,
            "bodySize": 4
        },
        "timings": {"send": 1.0, "wait": 8.0, "receive": 1.0},
        "serverIPAddress": "93.184.216.34",
        "connection": "443",
        "pageref": "page_1",
        "_webSocketMessages": [
            {"type": "send", "time": 1704067202.5, "opcode": 1, "data": "ping"},
            {"type": "receive", "time": 1704067202.7, "opcode": 1, "data": "pong"}
        ]
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

    #[test]
    fn deserialize_extended_fields() {
        let e: Entry = serde_json::from_str(ENTRY_WITH_EXTRAS).unwrap();
        assert_eq!(e.request.query_string[0].name, "w");
        assert_eq!(e.request.cookies[0].name, "session");
        assert_eq!(e.request.headers_size, 120);
        assert_eq!(e.response.cookies[0].http_only, Some(true));
        assert_eq!(e.response.redirect_url, "https://cdn.example.com/img.png");
        assert_eq!(e.response.body_size, 4);
        assert_eq!(e.response.content.encoding.as_deref(), Some("base64"));
        assert_eq!(e.server_ip_address.as_deref(), Some("93.184.216.34"));
        assert_eq!(e.pageref.as_deref(), Some("page_1"));
        let ws = e.websocket_messages.as_ref().unwrap();
        assert_eq!(ws[0].message_type, "send");
        assert_eq!(ws[1].data, "pong");
    }

    #[test]
    fn extended_fields_default_when_absent() {
        let e: Entry = serde_json::from_str(MINIMAL_ENTRY).unwrap();
        assert!(e.request.query_string.is_empty());
        assert!(e.request.cookies.is_empty());
        assert_eq!(e.response.redirect_url, "");
        assert!(e.response.content.encoding.is_none());
        assert!(e.server_ip_address.is_none());
        assert!(e.websocket_messages.is_none());
    }
}
