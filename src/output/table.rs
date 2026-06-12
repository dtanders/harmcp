use crate::cli::Column;
use crate::har::types::{Cookie, DecodedBody, Entry};

const URL_MAX: usize = 60;

pub fn print_list_header(columns: &[Column]) {
    let header = format_row(columns, column_header);
    println!("{}", header);
    println!("{}", "-".repeat(header.trim_end().len()));
}

pub fn print_list_row(entry: &Entry, index: usize, columns: &[Column]) {
    println!("{}", format_list_row(entry, index, columns));
}

pub fn format_list_row(entry: &Entry, index: usize, columns: &[Column]) -> String {
    format_row(columns, |c| column_value(c, entry, index))
}

fn format_row<F: Fn(&Column) -> String>(columns: &[Column], value: F) -> String {
    columns
        .iter()
        .map(|c| {
            let w = column_width(c);
            let v = value(c);
            let char_len = v.chars().count();
            let cell = if char_len > w {
                let truncated: String = v.chars().take(w.saturating_sub(3)).collect();
                format!("{}...", truncated)
            } else {
                v
            };
            format!("{:<width$}", cell, width = w)
        })
        .collect::<Vec<_>>()
        .join("  ")
}

fn column_header(col: &Column) -> String {
    match col {
        Column::Index => "IDX".to_string(),
        Column::Method => "METHOD".to_string(),
        Column::Status => "STATUS".to_string(),
        Column::Url => "URL".to_string(),
        Column::Mime => "MIME".to_string(),
        Column::Size => "SIZE".to_string(),
        Column::Time => "TIME(ms)".to_string(),
        Column::Start => "STARTED".to_string(),
    }
}

fn column_width(col: &Column) -> usize {
    match col {
        Column::Index => 5,
        Column::Method => 7,
        Column::Status => 6,
        Column::Url => URL_MAX,
        Column::Mime => 30,
        Column::Size => 10,
        Column::Time => 9,
        Column::Start => 30,
    }
}

fn column_value(col: &Column, entry: &Entry, index: usize) -> String {
    match col {
        Column::Index => index.to_string(),
        Column::Method => entry.request.method.clone(),
        Column::Status => entry.response.status.to_string(),
        Column::Url => entry.request.url.clone(),
        Column::Mime => entry.response.content.mime_type.clone(),
        Column::Size => entry.response.content.size.to_string(),
        Column::Time => format!("{:.1}", entry.time),
        Column::Start => entry.started_date_time.clone(),
    }
}

pub fn print_detail_headers(entry: &Entry) {
    println!("=== Request Headers ===");
    if entry.request.headers.is_empty() {
        println!("(none)");
    } else {
        for h in &entry.request.headers {
            println!("{}: {}", h.name, h.value);
        }
    }
    println!();
    println!("=== Response Headers ===");
    if entry.response.headers.is_empty() {
        println!("(none)");
    } else {
        for h in &entry.response.headers {
            println!("{}: {}", h.name, h.value);
        }
    }
}

pub fn print_detail_info(entry: &Entry) {
    println!("started:       {}", entry.started_date_time);
    println!(
        "status:        {} {}",
        entry.response.status, entry.response.status_text
    );
    if let Some(ip) = &entry.server_ip_address {
        println!("server ip:     {}", ip);
    }
    if let Some(c) = &entry.connection {
        println!("connection:    {}", c);
    }
    if let Some(p) = &entry.pageref {
        println!("page:          {}", p);
    }
    if !entry.response.redirect_url.is_empty() {
        println!("redirect to:   {}", entry.response.redirect_url);
    }
    println!("req headers:   {}", fmt_size(entry.request.headers_size));
    println!("req body:      {}", fmt_size(entry.request.body_size));
    println!("resp headers:  {}", fmt_size(entry.response.headers_size));
    println!(
        "resp body:     {}  (on the wire)",
        fmt_size(entry.response.body_size)
    );
    println!(
        "content size:  {}  (decompressed)",
        fmt_size(entry.response.content.size)
    );
    if !entry.request.query_string.is_empty() {
        println!();
        println!("=== Query Parameters ===");
        for q in &entry.request.query_string {
            println!("{} = {}", q.name, q.value);
        }
    }
}

fn fmt_size(size: i64) -> String {
    if size < 0 {
        "unknown".to_string()
    } else {
        format!("{} bytes", size)
    }
}

pub fn print_detail_cookies(entry: &Entry) {
    println!("=== Request Cookies ===");
    print_cookie_list(&entry.request.cookies);
    println!();
    println!("=== Response Cookies ===");
    print_cookie_list(&entry.response.cookies);
}

fn print_cookie_list(cookies: &[Cookie]) {
    if cookies.is_empty() {
        println!("(none)");
        return;
    }
    for c in cookies {
        let mut attrs = Vec::new();
        if let Some(d) = &c.domain {
            attrs.push(format!("domain={d}"));
        }
        if let Some(p) = &c.path {
            attrs.push(format!("path={p}"));
        }
        if let Some(e) = &c.expires {
            attrs.push(format!("expires={e}"));
        }
        if c.http_only == Some(true) {
            attrs.push("httpOnly".to_string());
        }
        if c.secure == Some(true) {
            attrs.push("secure".to_string());
        }
        let suffix = if attrs.is_empty() {
            String::new()
        } else {
            format!("  [{}]", attrs.join("; "))
        };
        println!("{}: {}{}", c.name, c.value, suffix);
    }
}

pub fn print_detail_body(entry: &Entry) {
    println!("=== Request Body ===");
    match entry
        .request
        .post_data
        .as_ref()
        .and_then(|p| p.text.as_deref())
    {
        Some(text) => println!("{}", text),
        None => println!("(none)"),
    }
    println!();
    println!("=== Response Body ===");
    match entry.response.content.decoded_body() {
        DecodedBody::None => println!("(none)"),
        DecodedBody::Text(t) => println!("{}", t),
        DecodedBody::DecodedText(s) => {
            println!("(decoded from base64)");
            println!("{}", s);
        }
        DecodedBody::Binary(b) => println!(
            "(binary body: {} bytes after base64 decode; use `body <idx> --output <file>` to save)",
            b.len()
        ),
        DecodedBody::Invalid(t) => {
            println!("(marked base64 but failed to decode; raw text follows)");
            println!("{}", t);
        }
    }
}

pub fn print_detail_timings(entry: &Entry) {
    let t = &entry.timings;
    print_opt_timing("blocked", t.blocked);
    print_opt_timing("dns", t.dns);
    print_opt_timing("connect", t.connect);
    println!("{:<12} {:>10.1} ms", "send", t.send);
    println!("{:<12} {:>10.1} ms", "wait", t.wait);
    println!("{:<12} {:>10.1} ms", "receive", t.receive);
    println!("{}", "-".repeat(26));
    println!("{:<12} {:>10.1} ms", "total", entry.time);
}

fn print_opt_timing(label: &str, value: Option<f64>) {
    match value {
        Some(v) if v >= 0.0 => println!("{:<12} {:>10.1} ms", label, v),
        _ => println!("{:<12} {:>10}", label, "n/a"),
    }
}

pub fn print_detail_stack(entry: &Entry) {
    match &entry.initiator {
        None => println!("(no initiator data)"),
        Some(init) => {
            println!("type: {}", init.initiator_type);
            match &init.stack {
                None => println!("(no call stack)"),
                Some(stack) => {
                    for frame in &stack.call_frames {
                        let name = if frame.function_name.is_empty() {
                            "(anonymous)"
                        } else {
                            &frame.function_name
                        };
                        println!("  {}  {}:{}", name, frame.url, frame.line_number);
                    }
                }
            }
        }
    }
}

pub fn print_detail_ws(entry: &Entry) {
    match entry.websocket_messages.as_deref() {
        None | Some([]) => println!("(no websocket messages)"),
        Some(msgs) => {
            for m in msgs {
                let dir = match m.message_type.as_str() {
                    "send" => "-> send",
                    "receive" => "<- recv",
                    other => other,
                };
                println!(
                    "[{:>14.3}] {} (opcode {}): {}",
                    m.time, dir, m.opcode, m.data
                );
            }
        }
    }
}

pub fn print_detail_all(entry: &Entry, index: usize) {
    println!(
        "Entry {}: {} {} => {} {}",
        index,
        entry.request.method,
        entry.request.url,
        entry.response.status,
        entry.response.status_text
    );
    println!();
    print_detail_info(entry);
    println!();
    print_detail_headers(entry);
    println!();
    print_detail_cookies(entry);
    println!();
    print_detail_body(entry);
    println!();
    print_detail_timings(entry);
    println!();
    print_detail_stack(entry);
    if entry
        .websocket_messages
        .as_deref()
        .is_some_and(|m| !m.is_empty())
    {
        println!();
        println!("=== WebSocket Messages ===");
        print_detail_ws(entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Column;
    use crate::har::types::*;

    fn sample_entry() -> Entry {
        let mut e = crate::har::types::test_entry();
        e.time = 55.5;
        e.request.method = "POST".to_string();
        e.request.url = "https://example.com/submit".to_string();
        e.request.headers = vec![Header {
            name: "Authorization".to_string(),
            value: "Bearer tok".to_string(),
        }];
        e.request.post_data = Some(PostData {
            text: Some(r#"{"x":1}"#.to_string()),
        });
        e.response.status = 201;
        e.response.status_text = "Created".to_string();
        e.response.headers = vec![Header {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        }];
        e.response.content.size = 128;
        e.response.content.text = Some(r#"{"id":42}"#.to_string());
        e.timings = Timings {
            blocked: Some(1.0),
            dns: Some(2.0),
            connect: Some(3.0),
            send: 1.0,
            wait: 50.0,
            receive: 4.5,
        };
        e.initiator = Some(Initiator {
            initiator_type: "script".to_string(),
            stack: Some(Stack {
                call_frames: vec![CallFrame {
                    function_name: "submitForm".to_string(),
                    url: "https://example.com/app.js".to_string(),
                    line_number: 99,
                    column_number: Some(5),
                }],
            }),
        });
        e
    }

    #[test]
    fn list_row_contains_method_and_status() {
        let s = format_list_row(&sample_entry(), 3, &Column::defaults());
        assert!(s.contains("POST"));
        assert!(s.contains("201"));
        assert!(s.contains("3"));
    }

    #[test]
    fn list_row_truncates_long_url() {
        let mut e = sample_entry();
        e.request.url = "https://example.com/".to_string() + &"a".repeat(100);
        let s = format_list_row(&e, 0, &[Column::Url]);
        assert!(s.trim().ends_with("..."), "expected truncation, got: {s}");
    }
}
