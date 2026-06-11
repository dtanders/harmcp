use std::path::Path;

use crate::cli::OutputFormat;
use crate::error::{HarError, Result};
use crate::har::stream::stream_entries;
use crate::input;
use crate::output::{self, DetailSection};

pub fn run(
    file: &Path,
    targets: &[usize],
    section: DetailSection,
    format: &OutputFormat,
) -> Result<()> {
    let mut wanted: Vec<usize> = targets.to_vec();
    wanted.sort_unstable();
    wanted.dedup();
    let max = *wanted.last().expect("clap guarantees at least one index");
    let mut found = 0usize;
    let mut first = true;
    let reader = input::open(file)?;
    let total = stream_entries(reader, |idx, entry| {
        if wanted.binary_search(&idx).is_ok() {
            if !first {
                println!();
            }
            first = false;
            output::print_detail(format, &entry, idx, section);
            found += 1;
        }
        Ok(idx < max)
    })?;
    if found < wanted.len() {
        let missing = wanted.iter().copied().find(|&i| i >= total).unwrap_or(max);
        return Err(HarError::IndexOutOfRange {
            index: missing,
            total,
        });
    }
    Ok(())
}
