use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::cli::OutputFormat;
use crate::error::{HarError, Result};
use crate::har::stream::stream_entries;
use crate::output::{self, DetailSection};

pub fn run(
    file: &Path,
    target: usize,
    section: DetailSection,
    format: &OutputFormat,
) -> Result<()> {
    let mut found = false;
    let reader = BufReader::new(File::open(file)?);
    let total = stream_entries(reader, |idx, entry| {
        if idx == target {
            found = true;
            output::print_detail(format, &entry, idx, section);
            return Ok(false);
        }
        Ok(true)
    })?;
    if !found {
        return Err(HarError::IndexOutOfRange { index: target, total });
    }
    Ok(())
}
