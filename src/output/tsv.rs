use crate::cli::Column;
use crate::har::types::Entry;

pub fn list_header(columns: &[Column]) -> String {
    columns.iter().map(column_name).collect::<Vec<_>>().join("\t")
}

pub fn list_row(entry: &Entry, index: usize, columns: &[Column]) -> String {
    columns
        .iter()
        .map(|c| column_value(c, entry, index))
        .collect::<Vec<_>>()
        .join("\t")
}

fn column_name(col: &Column) -> &'static str {
    match col {
        Column::Index => "index",
        Column::Method => "method",
        Column::Status => "status",
        Column::Url => "url",
        Column::Mime => "mime",
        Column::Size => "size",
        Column::Time => "time_ms",
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Column;
    use crate::har::types::*;

    fn sample_entry() -> Entry {
        Entry {
            started_date_time: String::new(),
            time: 123.4,
            request: Request {
                method: "GET".to_string(),
                url: "https://example.com/api".to_string(),
                headers: vec![],
                post_data: None,
            },
            response: Response {
                status: 200,
                status_text: "OK".to_string(),
                headers: vec![],
                content: Content {
                    size: 512,
                    mime_type: "application/json".to_string(),
                    text: None,
                },
            },
            timings: Timings {
                blocked: None,
                dns: None,
                connect: None,
                send: 0.5,
                wait: 100.0,
                receive: 16.9,
            },
            initiator: None,
        }
    }

    #[test]
    fn header_tab_count_matches_columns() {
        let cols = Column::defaults();
        let h = list_header(&cols);
        assert_eq!(h.split('\t').count(), cols.len());
    }

    #[test]
    fn row_fields_match_entry() {
        let entry = sample_entry();
        let cols = Column::defaults();
        let row = list_row(&entry, 7, &cols);
        let fields: Vec<&str> = row.split('\t').collect();
        assert_eq!(fields[0], "7");
        assert_eq!(fields[1], "GET");
        assert_eq!(fields[2], "200");
        assert_eq!(fields[3], "https://example.com/api");
        assert_eq!(fields[4], "application/json");
        assert_eq!(fields[5], "512");
        assert!(fields[6].parse::<f64>().is_ok());
    }
}
