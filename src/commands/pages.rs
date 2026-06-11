use std::path::Path;

use serde_json::json;

use crate::cli::OutputFormat;
use crate::error::Result;
use crate::har::stream::stream_pages;

pub fn run(file: &Path, format: &OutputFormat) -> Result<()> {
    let reader = crate::input::open(file)?;
    let pages = stream_pages(reader)?;
    match format {
        OutputFormat::Json => {
            for p in &pages {
                println!(
                    "{}",
                    json!({"id": p.id, "title": p.title, "startedDateTime": p.started_date_time})
                );
            }
        }
        OutputFormat::Tsv => {
            println!("id\tstarted\ttitle");
            for p in &pages {
                println!("{}\t{}\t{}", p.id, p.started_date_time, p.title);
            }
        }
        OutputFormat::Table => {
            if pages.is_empty() {
                println!("(no pages)");
                return Ok(());
            }
            for p in &pages {
                println!("{}  {}  {}", p.id, p.started_date_time, p.title);
            }
        }
    }
    Ok(())
}
