use serde_json::{json, Value};

use crate::cli::Column;
use crate::har::types::{Cookie, DecodedBody, Entry};

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

fn cookie_json(c: &Cookie) -> Value {
    json!({
        "name": c.name,
        "value": c.value,
        "domain": c.domain,
        "path": c.path,
        "expires": c.expires,
        "httpOnly": c.http_only,
        "secure": c.secure,
    })
}

pub fn cookies_value(entry: &Entry) -> Value {
    json!({
        "requestCookies": entry.request.cookies.iter().map(cookie_json).collect::<Vec<_>>(),
        "responseCookies": entry.response.cookies.iter().map(cookie_json).collect::<Vec<_>>(),
    })
}

pub fn info_value(entry: &Entry) -> Value {
    json!({
        "started": entry.started_date_time,
        "status": entry.response.status,
        "statusText": entry.response.status_text,
        "serverIPAddress": entry.server_ip_address,
        "connection": entry.connection,
        "pageref": entry.pageref,
        "redirectURL": if entry.response.redirect_url.is_empty() { Value::Null } else { json!(entry.response.redirect_url) },
        "requestHeadersSize": entry.request.headers_size,
        "requestBodySize": entry.request.body_size,
        "responseHeadersSize": entry.response.headers_size,
        "responseBodySize": entry.response.body_size,
        "contentSize": entry.response.content.size,
        "queryString": entry.request.query_string.iter()
            .map(|q| json!({"name": q.name, "value": q.value}))
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

pub fn ws_value(entry: &Entry) -> Value {
    match &entry.websocket_messages {
        None => json!(null),
        Some(msgs) => json!(msgs
            .iter()
            .map(|m| json!({
                "type": m.message_type,
                "time": m.time,
                "opcode": m.opcode,
                "data": m.data,
            }))
            .collect::<Vec<_>>()),
    }
}

pub fn all_value(entry: &Entry, index: usize) -> Value {
    json!({
        "index": index,
        "method": entry.request.method,
        "url": entry.request.url,
        "status": entry.response.status,
        "statusText": entry.response.status_text,
        "info": info_value(entry),
        "headers": headers_value(entry),
        "cookies": cookies_value(entry),
        "body": body_value(entry),
        "timings": timings_value(entry),
        "stack": stack_value(entry),
        "webSocketMessages": ws_value(entry),
    })
}
