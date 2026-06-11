use serde_json::{json, Value};

use crate::cli::Column;
use crate::har::types::{DecodedBody, Entry};

pub fn list_row_value(entry: &Entry, index: usize, columns: &[Column]) -> Value {
    let mut map = serde_json::Map::new();
    for col in columns {
        let (k, v) = match col {
            Column::Index => ("index", json!(index)),
            Column::Method => ("method", json!(entry.request.method)),
            Column::Status => ("status", json!(entry.response.status)),
            Column::Url => ("url", json!(entry.request.url)),
            Column::Mime => ("mime", json!(entry.response.content.mime_type)),
            Column::Size => ("size", json!(entry.response.content.size)),
            Column::Time => ("time_ms", json!(entry.time)),
        };
        map.insert(k.to_string(), v);
    }
    Value::Object(map)
}

pub fn headers_value(entry: &Entry) -> Value {
    json!({
        "requestHeaders": entry.request.headers.iter()
            .map(|h| json!({"name": h.name, "value": h.value}))
            .collect::<Vec<_>>(),
        "responseHeaders": entry.response.headers.iter()
            .map(|h| json!({"name": h.name, "value": h.value}))
            .collect::<Vec<_>>(),
    })
}

pub fn body_value(entry: &Entry) -> Value {
    let (response_body, note) = match entry.response.content.decoded_body() {
        DecodedBody::None => (Value::Null, Value::Null),
        DecodedBody::Text(t) => (json!(t), Value::Null),
        DecodedBody::DecodedText(s) => (json!(s), json!("decoded from base64")),
        DecodedBody::Binary(b) => (
            Value::Null,
            json!(format!(
                "binary body: {} bytes after base64 decode",
                b.len()
            )),
        ),
        DecodedBody::Invalid(t) => (json!(t), json!("marked base64 but failed to decode")),
    };
    json!({
        "requestBody": entry.request.post_data.as_ref().and_then(|p| p.text.as_deref()),
        "responseBody": response_body,
        "responseBodyNote": note,
    })
}

pub fn timings_value(entry: &Entry) -> Value {
    let t = &entry.timings;
    json!({
        "blocked": t.blocked,
        "dns": t.dns,
        "connect": t.connect,
        "send": t.send,
        "wait": t.wait,
        "receive": t.receive,
        "total": entry.time,
    })
}

pub fn stack_value(entry: &Entry) -> Value {
    match &entry.initiator {
        None => json!(null),
        Some(init) => json!({
            "type": init.initiator_type,
            "callFrames": init.stack.as_ref().map(|s| {
                s.call_frames.iter().map(|f| json!({
                    "functionName": f.function_name,
                    "url": f.url,
                    "lineNumber": f.line_number,
                    "columnNumber": f.column_number,
                })).collect::<Vec<_>>()
            }).unwrap_or_default(),
        }),
    }
}

pub fn all_value(entry: &Entry, index: usize) -> Value {
    json!({
        "index": index,
        "method": entry.request.method,
        "url": entry.request.url,
        "status": entry.response.status,
        "headers": headers_value(entry),
        "body": body_value(entry),
        "timings": timings_value(entry),
        "stack": stack_value(entry),
    })
}
