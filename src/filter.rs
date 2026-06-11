use regex::Regex;

use crate::cli::ListArgs;
use crate::error::Result;
use crate::har::types::Entry;

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
}

impl Filters {
    pub fn from_args(args: &ListArgs) -> Result<Self> {
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
    use crate::cli::ListArgs;
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

    fn no_filters() -> ListArgs {
        ListArgs {
            columns: None,
            method: None,
            status: None,
            url: None,
            url_regex: None,
            mime: None,
            min_size: None,
            max_size: None,
            no_media: false,
            no_css: false,
            no_assets: false,
        }
    }

    #[test]
    fn no_filters_matches_all() {
        let entry = make_entry("GET", "https://a.com", 200, "text/html", 100);
        let f = Filters::from_args(&no_filters()).unwrap();
        assert!(f.matches(&entry));
    }

    #[test]
    fn method_filter_case_insensitive() {
        let entry = make_entry("GET", "https://a.com", 200, "text/html", 100);
        let f = Filters::from_args(&ListArgs {
            method: Some("get".into()),
            ..no_filters()
        })
        .unwrap();
        assert!(f.matches(&entry));
        let f2 = Filters::from_args(&ListArgs {
            method: Some("POST".into()),
            ..no_filters()
        })
        .unwrap();
        assert!(!f2.matches(&entry));
    }

    #[test]
    fn status_exact_match() {
        let entry = make_entry("GET", "", 200, "", 0);
        let f = Filters::from_args(&ListArgs {
            status: Some("200".into()),
            ..no_filters()
        })
        .unwrap();
        assert!(f.matches(&entry));
        let f2 = Filters::from_args(&ListArgs {
            status: Some("404".into()),
            ..no_filters()
        })
        .unwrap();
        assert!(!f2.matches(&entry));
    }

    #[test]
    fn status_wildcard_4xx() {
        let ok = make_entry("GET", "", 200, "", 0);
        let err = make_entry("GET", "", 404, "", 0);
        let f = Filters::from_args(&ListArgs {
            status: Some("4xx".into()),
            ..no_filters()
        })
        .unwrap();
        assert!(!f.matches(&ok));
        assert!(f.matches(&err));
    }

    #[test]
    fn status_range() {
        let entry = make_entry("GET", "", 201, "", 0);
        let f = Filters::from_args(&ListArgs {
            status: Some("200-299".into()),
            ..no_filters()
        })
        .unwrap();
        assert!(f.matches(&entry));
        let err = make_entry("GET", "", 404, "", 0);
        assert!(!f.matches(&err));
    }

    #[test]
    fn url_substring_case_insensitive() {
        let entry = make_entry("GET", "https://EXAMPLE.com/API/users", 200, "", 0);
        let f = Filters::from_args(&ListArgs {
            url: Some("api".into()),
            ..no_filters()
        })
        .unwrap();
        assert!(f.matches(&entry));
    }

    #[test]
    fn mime_substring() {
        let entry = make_entry("GET", "", 200, "application/json; charset=utf-8", 0);
        let f = Filters::from_args(&ListArgs {
            mime: Some("json".into()),
            ..no_filters()
        })
        .unwrap();
        assert!(f.matches(&entry));
    }

    #[test]
    fn url_regex_matches_and_rejects() {
        let entry = make_entry("GET", "https://example.com/api/v2/users", 200, "", 0);
        let f = Filters::from_args(&ListArgs {
            url_regex: Some(r"api/v\d+/".into()),
            ..no_filters()
        })
        .unwrap();
        assert!(f.matches(&entry));
        let f2 = Filters::from_args(&ListArgs {
            url_regex: Some(r"^https://other\.com".into()),
            ..no_filters()
        })
        .unwrap();
        assert!(!f2.matches(&entry));
    }

    #[test]
    fn no_media_excludes_image_video_audio_font() {
        let f = Filters::from_args(&ListArgs {
            no_media: true,
            ..no_filters()
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
        let f = Filters::from_args(&ListArgs {
            no_css: true,
            ..no_filters()
        })
        .unwrap();
        assert!(!f.matches(&make_entry("GET", "", 200, "text/css", 0)));
        assert!(!f.matches(&make_entry("GET", "", 200, "text/css; charset=utf-8", 0)));
        assert!(f.matches(&make_entry("GET", "", 200, "application/json", 0)));
        assert!(f.matches(&make_entry("GET", "", 200, "image/png", 0)));
    }

    #[test]
    fn no_assets_excludes_both_media_and_css() {
        let f = Filters::from_args(&ListArgs {
            no_assets: true,
            ..no_filters()
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
        let fmin = Filters::from_args(&ListArgs {
            min_size: Some(100),
            ..no_filters()
        })
        .unwrap();
        let fmax = Filters::from_args(&ListArgs {
            max_size: Some(100),
            ..no_filters()
        })
        .unwrap();
        assert!(fmin.matches(&entry));
        assert!(!fmax.matches(&entry));
    }
}
