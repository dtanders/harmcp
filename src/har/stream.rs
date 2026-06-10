use std::fmt;
use std::io::Read;
use serde::de::{DeserializeSeed, Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor};

use crate::error::Result;
use crate::har::types::Entry;

/// Call `f(index, entry)` for each entry in the HAR `reader`.
/// `f` returns `Ok(true)` to continue, `Ok(false)` to stop early.
/// Returns the count of entries passed to `f`.
pub fn stream_entries<R, F>(reader: R, mut f: F) -> Result<usize>
where
    R: Read,
    F: FnMut(usize, Entry) -> Result<bool>,
{
    let mut count = 0usize;
    let mut de = serde_json::Deserializer::from_reader(reader);
    de.deserialize_map(HarRootVisitor {
        f: &mut f,
        count: &mut count,
    })?;
    Ok(count)
}

struct HarRootVisitor<'a, F> {
    f: &'a mut F,
    count: &'a mut usize,
}

impl<'a, 'de, F> Visitor<'de> for HarRootVisitor<'a, F>
where
    F: FnMut(usize, Entry) -> Result<bool>,
{
    type Value = ();
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("a HAR root object")
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> std::result::Result<(), A::Error> {
        while let Some(key) = map.next_key::<String>()? {
            if key == "log" {
                return map.next_value_seed(LogSeed {
                    f: self.f,
                    count: self.count,
                });
            }
            map.next_value::<IgnoredAny>()?;
        }
        Err(serde::de::Error::custom(
            "not a HAR file: missing top-level \"log\" key",
        ))
    }
}

struct LogSeed<'a, F> {
    f: &'a mut F,
    count: &'a mut usize,
}

impl<'a, 'de, F> DeserializeSeed<'de> for LogSeed<'a, F>
where
    F: FnMut(usize, Entry) -> Result<bool>,
{
    type Value = ();
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> std::result::Result<(), D::Error> {
        d.deserialize_map(LogVisitor {
            f: self.f,
            count: self.count,
        })
    }
}

struct LogVisitor<'a, F> {
    f: &'a mut F,
    count: &'a mut usize,
}

impl<'a, 'de, F> Visitor<'de> for LogVisitor<'a, F>
where
    F: FnMut(usize, Entry) -> Result<bool>,
{
    type Value = ();
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("a HAR log object")
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> std::result::Result<(), A::Error> {
        while let Some(key) = map.next_key::<String>()? {
            if key == "entries" {
                return map.next_value_seed(EntriesSeed {
                    f: self.f,
                    count: self.count,
                });
            }
            map.next_value::<IgnoredAny>()?;
        }
        Err(serde::de::Error::custom(
            "not a HAR file: missing \"entries\" in log",
        ))
    }
}

struct EntriesSeed<'a, F> {
    f: &'a mut F,
    count: &'a mut usize,
}

impl<'a, 'de, F> DeserializeSeed<'de> for EntriesSeed<'a, F>
where
    F: FnMut(usize, Entry) -> Result<bool>,
{
    type Value = ();
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> std::result::Result<(), D::Error> {
        d.deserialize_seq(EntriesVisitor {
            f: self.f,
            count: self.count,
        })
    }
}

struct EntriesVisitor<'a, F> {
    f: &'a mut F,
    count: &'a mut usize,
}

impl<'a, 'de, F> Visitor<'de> for EntriesVisitor<'a, F>
where
    F: FnMut(usize, Entry) -> Result<bool>,
{
    type Value = ();
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("an array of HAR entries")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> std::result::Result<(), A::Error> {
        let mut stopped = false;
        loop {
            if stopped {
                // Drain remaining elements cheaply without full deserialization.
                if seq.next_element::<IgnoredAny>()?.is_none() {
                    break;
                }
            } else {
                match seq.next_element::<serde_json::Value>()? {
                    None => break,
                    Some(value) => match serde_json::from_value::<Entry>(value) {
                        Err(e) => {
                            eprintln!("warning: skipping malformed entry: {e}");
                        }
                        Ok(entry) => {
                            let idx = *self.count;
                            *self.count += 1;
                            match (self.f)(idx, entry) {
                                Ok(true) => {}
                                Ok(false) => stopped = true,
                                Err(e) => return Err(serde::de::Error::custom(e)),
                            }
                        }
                    },
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TWO_ENTRY_HAR: &str = r#"{
        "log": {
            "version": "1.2",
            "entries": [
                {
                    "startedDateTime": "2024-01-01T00:00:00.000Z",
                    "time": 100.0,
                    "request": {"method": "GET", "url": "https://a.com/1", "headers": [], "headersSize": -1, "bodySize": -1},
                    "response": {"status": 200, "statusText": "OK", "headers": [], "content": {"size": 10, "mimeType": "text/plain"}, "redirectURL": "", "headersSize": -1, "bodySize": 10},
                    "timings": {"send": 1.0, "wait": 90.0, "receive": 9.0}
                },
                {
                    "startedDateTime": "2024-01-01T00:00:01.000Z",
                    "time": 200.0,
                    "request": {"method": "POST", "url": "https://a.com/2", "headers": [], "headersSize": -1, "bodySize": -1},
                    "response": {"status": 404, "statusText": "Not Found", "headers": [], "content": {"size": 0, "mimeType": "text/html"}, "redirectURL": "", "headersSize": -1, "bodySize": 0},
                    "timings": {"send": 1.0, "wait": 190.0, "receive": 9.0}
                }
            ]
        }
    }"#;

    #[test]
    fn streams_all_entries() {
        let mut urls = Vec::new();
        let count = stream_entries(TWO_ENTRY_HAR.as_bytes(), |_idx, entry| {
            urls.push(entry.request.url.clone());
            Ok(true)
        })
        .unwrap();
        assert_eq!(count, 2);
        assert_eq!(urls[0], "https://a.com/1");
        assert_eq!(urls[1], "https://a.com/2");
    }

    #[test]
    fn index_is_zero_based() {
        let mut indices = Vec::new();
        stream_entries(TWO_ENTRY_HAR.as_bytes(), |idx, _entry| {
            indices.push(idx);
            Ok(true)
        })
        .unwrap();
        assert_eq!(indices, vec![0, 1]);
    }

    #[test]
    fn early_stop_stops_callback() {
        let mut count = 0;
        stream_entries(TWO_ENTRY_HAR.as_bytes(), |_idx, _entry| {
            count += 1;
            Ok(false)
        })
        .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn invalid_json_returns_error() {
        let result = stream_entries(b"not json" as &[u8], |_, _| Ok(true));
        assert!(result.is_err());
    }

    #[test]
    fn missing_log_key_returns_error() {
        let result = stream_entries(br#"{"other": {}}"# as &[u8], |_, _| Ok(true));
        assert!(result.is_err());
    }
}
