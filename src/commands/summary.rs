use std::collections::HashMap;
use std::path::Path;

use serde_json::json;

use crate::cli::{FilterArgs, OutputFormat};
use crate::error::Result;
use crate::filter::Filters;
use crate::har::stream::stream_entries;

#[derive(Default)]
struct Agg {
    count: usize,
    total_bytes: i64,
    status: [usize; 6], // 1xx..5xx in [1..=5], other in [0]
    by_mime: HashMap<String, usize>,
    by_domain: HashMap<String, usize>,
    total_time: f64,
    slowest: Option<(usize, String, f64)>,
    largest: Option<(usize, String, i64)>,
    first_start: Option<String>,
    last_start: Option<String>,
}

pub fn run(file: &Path, args: &FilterArgs, format: &OutputFormat) -> Result<()> {
    let filters = Filters::from_args(args)?;
    let reader = crate::input::open(file)?;
    let mut agg = Agg::default();
    stream_entries(reader, |idx, entry| {
        if !filters.matches(&entry) {
            return Ok(true);
        }
        agg.count += 1;
        let size = entry.response.content.size.max(0);
        agg.total_bytes += size;
        let class = (entry.response.status / 100) as usize;
        agg.status[if (1..=5).contains(&class) { class } else { 0 }] += 1;
        *agg.by_mime
            .entry(mime_key(&entry.response.content.mime_type))
            .or_default() += 1;
        *agg.by_domain
            .entry(domain_of(&entry.request.url))
            .or_default() += 1;
        agg.total_time += entry.time;
        if agg.slowest.as_ref().is_none_or(|(_, _, t)| entry.time > *t) {
            agg.slowest = Some((idx, entry.request.url.clone(), entry.time));
        }
        if agg.largest.as_ref().is_none_or(|(_, _, s)| size > *s) {
            agg.largest = Some((idx, entry.request.url.clone(), size));
        }
        let start = &entry.started_date_time;
        if agg.first_start.as_ref().is_none_or(|f| start < f) {
            agg.first_start = Some(start.clone());
        }
        if agg.last_start.as_ref().is_none_or(|l| start > l) {
            agg.last_start = Some(start.clone());
        }
        Ok(true)
    })?;
    match format {
        OutputFormat::Table | OutputFormat::Tsv => print_table(&agg),
        OutputFormat::Json => print_json(&agg),
    }
    Ok(())
}

fn mime_key(mime: &str) -> String {
    let key = mime.split(';').next().unwrap_or(mime).trim();
    if key.is_empty() {
        "(none)".to_string()
    } else {
        key.to_string()
    }
}

fn domain_of(url: &str) -> String {
    let rest = url.split("://").nth(1).unwrap_or(url);
    let host_port = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    host_port.split(':').next().unwrap_or(host_port).to_string()
}

fn top5(map: &HashMap<String, usize>) -> Vec<(&String, &usize)> {
    let mut v: Vec<_> = map.iter().collect();
    v.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    v.truncate(5);
    v
}

fn print_table(agg: &Agg) {
    println!("entries:      {}", agg.count);
    println!("total bytes:  {}", agg.total_bytes);
    println!("total time:   {:.1} ms", agg.total_time);
    if let (Some(f), Some(l)) = (&agg.first_start, &agg.last_start) {
        println!("time span:    {} .. {}", f, l);
    }
    println!();
    println!(
        "status:  1xx {}   2xx {}   3xx {}   4xx {}   5xx {}   other {}",
        agg.status[1], agg.status[2], agg.status[3], agg.status[4], agg.status[5], agg.status[0]
    );
    println!();
    println!("top mime types:");
    for (mime, n) in top5(&agg.by_mime) {
        println!("  {:<40} {}", mime, n);
    }
    println!();
    println!("top domains:");
    for (domain, n) in top5(&agg.by_domain) {
        println!("  {:<40} {}", domain, n);
    }
    if let Some((idx, url, t)) = &agg.slowest {
        println!();
        println!("slowest:  [{}] {:.1} ms  {}", idx, t, url);
    }
    if let Some((idx, url, s)) = &agg.largest {
        println!("largest:  [{}] {} bytes  {}", idx, s, url);
    }
}

fn print_json(agg: &Agg) {
    let v = json!({
        "entries": agg.count,
        "totalBytes": agg.total_bytes,
        "totalTimeMs": agg.total_time,
        "firstStart": agg.first_start,
        "lastStart": agg.last_start,
        "statusClasses": {
            "1xx": agg.status[1], "2xx": agg.status[2], "3xx": agg.status[3],
            "4xx": agg.status[4], "5xx": agg.status[5], "other": agg.status[0],
        },
        "topMimeTypes": top5(&agg.by_mime).iter().map(|(k, n)| json!({"mime": k, "count": n})).collect::<Vec<_>>(),
        "topDomains": top5(&agg.by_domain).iter().map(|(k, n)| json!({"domain": k, "count": n})).collect::<Vec<_>>(),
        "slowest": agg.slowest.as_ref().map(|(i, u, t)| json!({"index": i, "url": u, "timeMs": t})),
        "largest": agg.largest.as_ref().map(|(i, u, s)| json!({"index": i, "url": u, "bytes": s})),
    });
    println!("{}", v);
}
