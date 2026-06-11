use std::cmp::Ordering;
use std::path::Path;

use crate::cli::{Column, ListArgs, OutputFormat};
use crate::error::Result;
use crate::filter::Filters;
use crate::har::stream::stream_entries;
use crate::har::types::Entry;
use crate::output;

pub fn run(file: &Path, args: &ListArgs, format: &OutputFormat) -> Result<()> {
    let columns = args.columns.clone().unwrap_or_else(Column::defaults);
    let filters = Filters::from_args(&args.filters)?;
    let reader = crate::input::open(file)?;
    output::print_list_header(format, &columns);

    match &args.sort {
        None => {
            let mut shown = 0usize;
            stream_entries(reader, |idx, entry| {
                if filters.matches(&entry) {
                    output::print_list_row(format, &entry, idx, &columns);
                    shown += 1;
                    if args.limit.is_some_and(|l| shown >= l) {
                        return Ok(false);
                    }
                }
                Ok(true)
            })?;
        }
        Some(sort_col) => {
            let mut rows: Vec<(usize, Entry)> = Vec::new();
            stream_entries(reader, |idx, mut entry| {
                if filters.matches(&entry) {
                    // Shed bodies before buffering — not shown in list output.
                    entry.response.content.text = None;
                    entry.request.post_data = None;
                    rows.push((idx, entry));
                }
                Ok(true)
            })?;
            if args.desc {
                rows.sort_by(|a, b| compare(sort_col, b, a));
            } else {
                rows.sort_by(|a, b| compare(sort_col, a, b));
            }
            if let Some(l) = args.limit {
                rows.truncate(l);
            }
            for (idx, entry) in &rows {
                output::print_list_row(format, entry, *idx, &columns);
            }
        }
    }
    Ok(())
}

fn compare(col: &Column, a: &(usize, Entry), b: &(usize, Entry)) -> Ordering {
    let ord = match col {
        Column::Index => a.0.cmp(&b.0),
        Column::Method => a.1.request.method.cmp(&b.1.request.method),
        Column::Status => a.1.response.status.cmp(&b.1.response.status),
        Column::Url => a.1.request.url.cmp(&b.1.request.url),
        Column::Mime => {
            a.1.response
                .content
                .mime_type
                .cmp(&b.1.response.content.mime_type)
        }
        Column::Size => a.1.response.content.size.cmp(&b.1.response.content.size),
        Column::Time => a.1.time.partial_cmp(&b.1.time).unwrap_or(Ordering::Equal),
        Column::Start => a.1.started_date_time.cmp(&b.1.started_date_time),
    };
    ord.then(a.0.cmp(&b.0))
}
