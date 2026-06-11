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
