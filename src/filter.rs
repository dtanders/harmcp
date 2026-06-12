use chrono::{DateTime, FixedOffset, NaiveDate};
use regex::Regex;

use crate::cli::FilterArgs;
use crate::error::Result;
use crate::har::types::{Entry, Header};

pub struct Filters {
    method: Option<String>,
    status: Option<String>,
    url: Option<String>,
    url_regex: Option<Regex>,
    mime: Option<String>,
    min_size: Option<i64>,
    max_size: Option<i64>,
    exclude_media: bool,
    exclude_css: bool,
    after: Option<DateTime<FixedOffset>>,
    before: Option<DateTime<FixedOffset>>,
    min_time: Option<f64>,
    max_time: Option<f64>,
    not_url: Option<String>,
    not_mime: Option<String>,
    not_status: Option<String>,
    not_method: Option<String>,
    header: Vec<(String, Option<String>)>,
    resp_header: Vec<(String, Option<String>)>,
    page: Option<String>,
}

impl Filters {
    pub fn from_args(args: &FilterArgs) -> Result<Self> {
        let url_regex = args.url_regex.as_deref().map(Regex::new).transpose()?;
        Ok(Self {
            method: args.method.clone(),
            status: args.status.clone(),
            url: args.url.as_deref().map(str::to_lowercase),
            url_regex,
            mime: args.mime.as_deref().map(str::to_lowercase),
            min_size: args.min_size,
            max_size: args.max_size,
            exclude_media: args.no_media || args.no_assets,
            exclude_css: args.no_css || args.no_assets,
            after: args.after.as_deref().map(parse_when).transpose()?,
            before: args.before.as_deref().map(parse_when).transpose()?,
            min_time: args.min_time,
            max_time: args.max_time,
            not_url: args.not_url.as_deref().map(str::to_lowercase),
            not_mime: args.not_mime.as_deref().map(str::to_lowercase),
            not_status: args.not_status.clone(),
            not_method: args.not_method.clone(),
            header: args
                .header
                .iter()
                .map(|s| parse_header_filter(s.as_str()))
                .collect(),
            resp_header: args
                .resp_header
                .iter()
                .map(|s| parse_header_filter(s.as_str()))
                .collect(),
            page: args.page.clone(),
        })
    }

    pub fn matches(&self, entry: &Entry) -> bool {
        if let Some(m) = &self.method {
            if !entry.request.method.eq_ignore_ascii_case(m) {
                return false;
            }
        }
        if let Some(s) = &self.status {
            if !matches_status(entry.response.status, s) {
                return false;
            }
        }
        if let Some(u) = &self.url {
            if !entry.request.url.to_lowercase().contains(u.as_str()) {
                return false;
            }
        }
        if let Some(re) = &self.url_regex {
            if !re.is_match(&entry.request.url) {
                return false;
            }
        }
        if let Some(m) = &self.mime {
            if !entry
                .response
                .content
                .mime_type
                .to_lowercase()
                .contains(m.as_str())
            {
                return false;
            }
        }
        if let Some(min) = self.min_size {
            if entry.response.content.size < min {
                return false;
            }
        }
        if let Some(max) = self.max_size {
            if entry.response.content.size > max {
                return false;
            }
        }
        if self.exclude_media && is_media_mime(&entry.response.content.mime_type) {
            return false;
        }
        if self.exclude_css && is_css_mime(&entry.response.content.mime_type) {
            return false;
        }
        if let Some(u) = &self.not_url {
            if entry.request.url.to_lowercase().contains(u.as_str()) {
                return false;
            }
        }
        if let Some(m) = &self.not_mime {
            if entry
                .response
                .content
                .mime_type
                .to_lowercase()
                .contains(m.as_str())
            {
                return false;
            }
        }
        if let Some(s) = &self.not_status {
            if matches_status(entry.response.status, s) {
                return false;
            }
        }
        if let Some(m) = &self.not_method {
            if entry.request.method.eq_ignore_ascii_case(m) {
                return false;
            }
        }
        if let Some(min) = self.min_time {
            if entry.time < min {
                return false;
            }
        }
        if let Some(max) = self.max_time {
            if entry.time > max {
                return false;
            }
        }
        if !headers_match(&entry.request.headers, &self.header) {
            return false;
        }
        if !headers_match(&entry.response.headers, &self.resp_header) {
            return false;
        }
        if let Some(p) = &self.page {
            if entry.pageref.as_deref() != Some(p.as_str()) {
                return false;
            }
        }
        if self.after.is_some() || self.before.is_some() {
            match DateTime::parse_from_rfc3339(&entry.started_date_time) {
                Err(_) => return false,
                Ok(started) => {
                    if let Some(a) = self.after {
                        if started < a {
                            return false;
                        }
                    }
                    if let Some(b) = self.before {
                        if started >= b {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }
}

fn is_media_mime(mime: &str) -> bool {
    let m = mime.to_ascii_lowercase();
    m.starts_with("image/")
        || m.starts_with("video/")
        || m.starts_with("audio/")
        || m.starts_with("font/")
        || m.starts_with("application/font-")
        || m.starts_with("application/x-font-")
}

fn is_css_mime(mime: &str) -> bool {
    mime.to_ascii_lowercase().starts_with("text/css")
}

fn parse_header_filter(s: &str) -> (String, Option<String>) {
    match s.split_once('=') {
        Some((name, value)) => (name.to_lowercase(), Some(value.to_lowercase())),
        None => (s.to_lowercase(), None),
    }
}

fn headers_match(headers: &[Header], wanted: &[(String, Option<String>)]) -> bool {
    wanted.iter().all(|(name, value)| {
        headers.iter().any(|h| {
            h.name.to_lowercase() == *name
                && value
                    .as_deref()
                    .is_none_or(|v| h.value.to_lowercase().contains(v))
        })
    })
}

fn parse_when(s: &str) -> crate::error::Result<DateTime<FixedOffset>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt);
    }
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let midnight = d.and_hms_opt(0, 0, 0).expect("00:00:00 is valid");
        return Ok(DateTime::from_naive_utc_and_offset(
            midnight,
            FixedOffset::east_opt(0).expect("UTC offset is valid"),
        ));
    }
    Err(crate::error::HarError::Usage(format!(
        "invalid date '{s}': use RFC 3339 (2024-01-01T12:00:00Z) or YYYY-MM-DD"
    )))
}

fn matches_status(status: u16, pattern: &str) -> bool {
    if let Ok(n) = pattern.parse::<u16>() {
        return status == n;
    }
    if let Some((lo, hi)) = pattern.split_once('-') {
        if let (Ok(lo), Ok(hi)) = (lo.parse::<u16>(), hi.parse::<u16>()) {
            return status >= lo && status <= hi;
        }
    }
    let s = status.to_string();
    if pattern.len() == s.len() {
        return pattern
            .chars()
            .zip(s.chars())
            .all(|(p, c)| matches!(p, 'x' | 'X') || p == c);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::FilterArgs;
    use crate::har::types::*;

    fn make_entry(method: &str, url: &str, status: u16, mime: &str, size: i64) -> Entry {
        let mut e = crate::har::types::test_entry();
        e.request.method = method.to_string();
        e.request.url = url.to_string();
        e.response.status = status;
        e.response.content.mime_type = mime.to_string();
        e.response.content.size = size;
        e
    }

    #[test]
    fn no_filters_matches_all() {
        let entry = make_entry("GET", "https://a.com", 200, "text/html", 100);
        let f = Filters::from_args(&FilterArgs::default()).unwrap();
        assert!(f.matches(&entry));
    }

    #[test]
    fn method_filter_case_insensitive() {
        let entry = make_entry("GET", "https://a.com", 200, "text/html", 100);
        let f = Filters::from_args(&FilterArgs {
            method: Some("get".into()),
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(f.matches(&entry));
        let f2 = Filters::from_args(&FilterArgs {
            method: Some("POST".into()),
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(!f2.matches(&entry));
    }

    #[test]
    fn status_exact_match() {
        let entry = make_entry("GET", "", 200, "", 0);
        let f = Filters::from_args(&FilterArgs {
            status: Some("200".into()),
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(f.matches(&entry));
        let f2 = Filters::from_args(&FilterArgs {
            status: Some("404".into()),
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(!f2.matches(&entry));
    }

    #[test]
    fn status_wildcard_4xx() {
        let ok = make_entry("GET", "", 200, "", 0);
        let err = make_entry("GET", "", 404, "", 0);
        let f = Filters::from_args(&FilterArgs {
            status: Some("4xx".into()),
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(!f.matches(&ok));
        assert!(f.matches(&err));
    }

    #[test]
    fn status_range() {
        let entry = make_entry("GET", "", 201, "", 0);
        let f = Filters::from_args(&FilterArgs {
            status: Some("200-299".into()),
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(f.matches(&entry));
        let err = make_entry("GET", "", 404, "", 0);
        assert!(!f.matches(&err));
    }

    #[test]
    fn url_substring_case_insensitive() {
        let entry = make_entry("GET", "https://EXAMPLE.com/API/users", 200, "", 0);
        let f = Filters::from_args(&FilterArgs {
            url: Some("api".into()),
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(f.matches(&entry));
    }

    #[test]
    fn mime_substring() {
        let entry = make_entry("GET", "", 200, "application/json; charset=utf-8", 0);
        let f = Filters::from_args(&FilterArgs {
            mime: Some("json".into()),
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(f.matches(&entry));
    }

    #[test]
    fn url_regex_matches_and_rejects() {
        let entry = make_entry("GET", "https://example.com/api/v2/users", 200, "", 0);
        let f = Filters::from_args(&FilterArgs {
            url_regex: Some(r"api/v\d+/".into()),
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(f.matches(&entry));
        let f2 = Filters::from_args(&FilterArgs {
            url_regex: Some(r"^https://other\.com".into()),
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(!f2.matches(&entry));
    }

    #[test]
    fn no_media_excludes_image_video_audio_font() {
        let f = Filters::from_args(&FilterArgs {
            no_media: true,
            ..FilterArgs::default()
        })
        .unwrap();
        for mime in &[
            "image/png",
            "image/svg+xml",
            "video/mp4",
            "audio/mpeg",
            "font/woff2",
            "application/font-woff",
            "application/x-font-ttf",
        ] {
            assert!(
                !f.matches(&make_entry("GET", "", 200, mime, 0)),
                "should exclude {mime}"
            );
        }
        assert!(f.matches(&make_entry("GET", "", 200, "application/json", 0)));
        assert!(f.matches(&make_entry("GET", "", 200, "text/css", 0)));
    }

    #[test]
    fn no_css_excludes_css_only() {
        let f = Filters::from_args(&FilterArgs {
            no_css: true,
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(!f.matches(&make_entry("GET", "", 200, "text/css", 0)));
        assert!(!f.matches(&make_entry("GET", "", 200, "text/css; charset=utf-8", 0)));
        assert!(f.matches(&make_entry("GET", "", 200, "application/json", 0)));
        assert!(f.matches(&make_entry("GET", "", 200, "image/png", 0)));
    }

    #[test]
    fn no_assets_excludes_both_media_and_css() {
        let f = Filters::from_args(&FilterArgs {
            no_assets: true,
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(!f.matches(&make_entry("GET", "", 200, "image/png", 0)));
        assert!(!f.matches(&make_entry("GET", "", 200, "text/css", 0)));
        assert!(f.matches(&make_entry("GET", "", 200, "application/json", 0)));
        assert!(f.matches(&make_entry("GET", "", 200, "text/html", 0)));
    }

    #[test]
    fn size_min_max() {
        let entry = make_entry("GET", "", 200, "", 500);
        let fmin = Filters::from_args(&FilterArgs {
            min_size: Some(100),
            ..FilterArgs::default()
        })
        .unwrap();
        let fmax = Filters::from_args(&FilterArgs {
            max_size: Some(100),
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(fmin.matches(&entry));
        assert!(!fmax.matches(&entry));
    }

    #[test]
    fn after_and_before_filter_by_started_date() {
        let mut early = make_entry("GET", "", 200, "", 0);
        early.started_date_time = "2024-01-01T00:00:00.000Z".to_string();
        let mut late = make_entry("GET", "", 200, "", 0);
        late.started_date_time = "2024-01-01T00:00:05.000Z".to_string();

        let f = Filters::from_args(&FilterArgs {
            after: Some("2024-01-01T00:00:02Z".into()),
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(!f.matches(&early));
        assert!(f.matches(&late));

        let f2 = Filters::from_args(&FilterArgs {
            before: Some("2024-01-01T00:00:02Z".into()),
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(f2.matches(&early));
        assert!(!f2.matches(&late));
    }

    #[test]
    fn date_only_shorthand_accepted() {
        let mut e = make_entry("GET", "", 200, "", 0);
        e.started_date_time = "2024-06-15T10:00:00.000Z".to_string();
        let f = Filters::from_args(&FilterArgs {
            after: Some("2024-06-01".into()),
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(f.matches(&e));
    }

    #[test]
    fn invalid_date_is_an_error() {
        assert!(Filters::from_args(&FilterArgs {
            after: Some("yesterday".into()),
            ..FilterArgs::default()
        })
        .is_err());
    }

    #[test]
    fn not_url_excludes_substring_match() {
        let telemetry = make_entry("GET", "https://example.com/telemetry/beacon", 200, "", 0);
        let api = make_entry("GET", "https://example.com/api/users", 200, "", 0);
        let f = Filters::from_args(&FilterArgs {
            not_url: Some("Telemetry".into()),
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(!f.matches(&telemetry));
        assert!(f.matches(&api));
    }

    #[test]
    fn not_mime_and_not_status_and_not_method() {
        let e = make_entry("POST", "https://a.com", 404, "text/html", 0);
        let f = Filters::from_args(&FilterArgs {
            not_mime: Some("html".into()),
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(!f.matches(&e));
        let f = Filters::from_args(&FilterArgs {
            not_status: Some("4xx".into()),
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(!f.matches(&e));
        let f = Filters::from_args(&FilterArgs {
            not_method: Some("post".into()),
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(!f.matches(&e));
        let ok = make_entry("GET", "https://a.com", 200, "application/json", 0);
        let f = Filters::from_args(&FilterArgs {
            not_mime: Some("html".into()),
            not_status: Some("4xx".into()),
            not_method: Some("post".into()),
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(f.matches(&ok));
    }

    #[test]
    fn min_max_time_filter() {
        let mut slow = make_entry("GET", "", 200, "", 0);
        slow.time = 2000.0;
        let mut fast = make_entry("GET", "", 200, "", 0);
        fast.time = 5.0;

        let f = Filters::from_args(&FilterArgs {
            min_time: Some(1000.0),
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(f.matches(&slow));
        assert!(!f.matches(&fast));

        let f2 = Filters::from_args(&FilterArgs {
            max_time: Some(1000.0),
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(!f2.matches(&slow));
        assert!(f2.matches(&fast));
    }

    #[test]
    fn header_filter_by_name_and_value() {
        let mut e = make_entry("GET", "https://a.com", 200, "", 0);
        e.request.headers = vec![Header {
            name: "Authorization".to_string(),
            value: "Bearer tok123".to_string(),
        }];

        let f = Filters::from_args(&FilterArgs {
            header: vec!["authorization".into()],
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(f.matches(&e));

        let f = Filters::from_args(&FilterArgs {
            header: vec!["authorization=bearer".into()],
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(f.matches(&e));

        let f = Filters::from_args(&FilterArgs {
            header: vec!["authorization=basic".into()],
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(!f.matches(&e));

        let f = Filters::from_args(&FilterArgs {
            header: vec!["x-missing".into()],
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(!f.matches(&e));
    }

    #[test]
    fn resp_header_filter() {
        let mut e = make_entry("GET", "https://a.com", 200, "", 0);
        e.response.headers = vec![Header {
            name: "Cache-Control".to_string(),
            value: "no-store".to_string(),
        }];
        let f = Filters::from_args(&FilterArgs {
            resp_header: vec!["cache-control=no-store".into()],
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(f.matches(&e));
        let f = Filters::from_args(&FilterArgs {
            resp_header: vec!["etag".into()],
            ..FilterArgs::default()
        })
        .unwrap();
        assert!(!f.matches(&e));
    }
}
