use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;

fn harmcp(args: &[&str]) -> assert_cmd::assert::Assert {
    let fixture = Path::new("tests/fixtures/sample.har");
    let mut cmd = Command::cargo_bin("harmcp").unwrap();
    cmd.arg(fixture);
    for arg in args {
        cmd.arg(arg);
    }
    cmd.assert()
}

#[test]
fn list_default_shows_both_entries() {
    harmcp(&["list"])
        .success()
        .stdout(predicate::str::contains("GET"))
        .stdout(predicate::str::contains("POST"))
        .stdout(predicate::str::contains("200"))
        .stdout(predicate::str::contains("401"));
}

#[test]
fn list_filter_by_method_post() {
    let out = harmcp(&["list", "--method", "POST"])
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    // Skip header lines (2), check only data rows
    let data_rows: Vec<&str> = s.lines().skip(2).collect();
    assert!(data_rows.iter().any(|l| l.contains("POST")));
    assert!(!data_rows.iter().any(|l| l.contains("GET")));
}

#[test]
fn list_filter_by_status_4xx() {
    let out = harmcp(&["list", "--status", "4xx"])
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let data_rows: Vec<&str> = s.lines().skip(2).collect();
    assert!(data_rows.iter().any(|l| l.contains("401")));
    assert!(!data_rows.iter().any(|l| l.contains("200")));
}

#[test]
fn list_filter_by_url_substring() {
    let out = harmcp(&["list", "--url", "users"])
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let data_rows: Vec<&str> = s.lines().skip(2).collect();
    assert!(data_rows.iter().any(|l| l.contains("users")));
    assert!(!data_rows.iter().any(|l| l.contains("login")));
}

#[test]
fn list_filter_by_mime() {
    let out = harmcp(&["list", "--mime", "json"])
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let data_rows: Vec<&str> = s.lines().skip(2).collect();
    assert!(data_rows.iter().any(|l| l.contains("application/json")));
    assert!(!data_rows.iter().any(|l| l.contains("text/html")));
}

#[test]
fn list_tsv_format_has_tabs() {
    let out = harmcp(&["--format", "tsv", "list"])
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.lines().next().unwrap().contains('\t'));
}

#[test]
fn list_json_format_each_line_is_json() {
    let out = harmcp(&["--format", "json", "list"])
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    for line in s.lines() {
        serde_json::from_str::<serde_json::Value>(line)
            .unwrap_or_else(|_| panic!("not valid JSON: {line}"));
    }
}

#[test]
fn headers_command_shows_accept_header() {
    harmcp(&["headers", "0"])
        .success()
        .stdout(predicate::str::contains("Accept"))
        .stdout(predicate::str::contains("application/json"));
}

#[test]
fn body_command_shows_response_body() {
    harmcp(&["body", "0"])
        .success()
        .stdout(predicate::str::contains(r#"[{"id":1}]"#));
}

#[test]
fn timings_command_shows_wait_and_total() {
    harmcp(&["timings", "0"])
        .success()
        .stdout(predicate::str::contains("wait"))
        .stdout(predicate::str::contains("total"));
}

#[test]
fn stack_command_shows_function_name() {
    harmcp(&["stack", "0"])
        .success()
        .stdout(predicate::str::contains("fetchUsers"));
}

#[test]
fn all_command_combines_sections() {
    harmcp(&["all", "0"])
        .success()
        .stdout(predicate::str::contains("GET"))
        .stdout(predicate::str::contains("Accept"))
        .stdout(predicate::str::contains("wait"))
        .stdout(predicate::str::contains("fetchUsers"));
}

#[test]
fn index_out_of_range_exits_nonzero_with_message() {
    harmcp(&["headers", "99"])
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn nonexistent_file_exits_nonzero() {
    Command::cargo_bin("harmcp")
        .unwrap()
        .arg("no_such_file.har")
        .arg("list")
        .assert()
        .failure();
}

#[test]
fn version_flag_prints_version() {
    Command::cargo_bin("harmcp")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn stdin_input_with_dash() {
    let har = std::fs::read_to_string("tests/fixtures/sample.har").unwrap();
    Command::cargo_bin("harmcp")
        .unwrap()
        .arg("-")
        .arg("list")
        .write_stdin(har)
        .assert()
        .success()
        .stdout(predicate::str::contains("api/users"));
}

#[test]
fn detail_accepts_multiple_indices() {
    harmcp(&["headers", "0", "1"])
        .success()
        .stdout(predicate::str::contains("Accept")) // entry 0 request header
        .stdout(predicate::str::contains("Content-Type")); // entry 1 request header
}

#[test]
fn detail_multiple_indices_one_missing_fails_after_printing_found() {
    harmcp(&["headers", "0", "99"])
        .failure()
        .stdout(predicate::str::contains("Accept"))
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn body_of_base64_binary_entry_shows_note_not_garbage() {
    harmcp(&["body", "2"])
        .success()
        .stdout(predicate::str::contains("binary body"))
        .stdout(predicate::str::contains("4 bytes"));
}

#[test]
fn body_output_writes_decoded_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("logo.png");
    Command::cargo_bin("harmcp")
        .unwrap()
        .arg("tests/fixtures/sample.har")
        .args(["body", "2", "--output"])
        .arg(&out)
        .assert()
        .success();
    let bytes = std::fs::read(&out).unwrap();
    assert_eq!(bytes.len(), 4); // "iVBORw==" decodes to 4 bytes
}

#[test]
fn body_output_rejects_multiple_indices() {
    harmcp(&["body", "0", "1", "--output", "x.bin"])
        .failure()
        .stderr(predicate::str::contains("single index"));
}

#[test]
fn cookies_command_shows_request_and_response_cookies() {
    harmcp(&["cookies", "0"])
        .success()
        .stdout(predicate::str::contains("session"))
        .stdout(predicate::str::contains("abc123"))
        .stdout(predicate::str::contains("httpOnly"));
}

#[test]
fn info_command_shows_metadata() {
    harmcp(&["info", "0"])
        .success()
        .stdout(predicate::str::contains("2024-01-01T00:00:00.000Z"))
        .stdout(predicate::str::contains("200 OK"))
        .stdout(predicate::str::contains("93.184.216.34"))
        .stdout(predicate::str::contains("page_1"))
        .stdout(predicate::str::contains("limit = 10"));
}

#[test]
fn ws_command_shows_messages() {
    harmcp(&["ws", "3"])
        .success()
        .stdout(predicate::str::contains("send"))
        .stdout(predicate::str::contains("subscribe"))
        .stdout(predicate::str::contains("recv"));
}

#[test]
fn ws_command_on_non_ws_entry_says_none() {
    harmcp(&["ws", "0"])
        .success()
        .stdout(predicate::str::contains("no websocket messages"));
}

#[test]
fn list_after_filter_and_start_column() {
    let out = harmcp(&[
        "list",
        "--after",
        "2024-01-01T00:00:01.500Z",
        "--columns",
        "index,url,start",
    ])
    .success()
    .get_output()
    .stdout
    .clone();
    let s = String::from_utf8(out).unwrap();
    let data_rows: Vec<&str> = s.lines().skip(2).collect();
    // entries 2 and 3 start at 00:00:02 and 00:00:03, which are after 00:00:01.500
    assert!(data_rows.iter().any(|l| l.contains("logo.png")));
    assert!(!data_rows.iter().any(|l| l.contains("api/users")));
    assert!(data_rows.iter().any(|l| l.contains("2024-01-01T00:00:02")));
}

#[test]
fn list_not_mime_excludes_json() {
    let out = harmcp(&["list", "--not-mime", "json"])
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let data_rows: Vec<&str> = s.lines().skip(2).collect();
    assert!(!data_rows.iter().any(|l| l.contains("application/json")));
    assert!(data_rows.iter().any(|l| l.contains("text/html")));
}
