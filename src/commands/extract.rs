use std::path::Path;

use crate::error::{HarError, Result};
use crate::har::stream::stream_entries;
use crate::har::types::DecodedBody;
use crate::input;

/// Write entry `target`'s decoded response body to `out`.
pub fn run(file: &Path, target: usize, out: &Path) -> Result<()> {
    let mut bytes: Option<Vec<u8>> = None;
    let reader = input::open(file)?;
    let total = stream_entries(reader, |idx, entry| {
        if idx == target {
            bytes = Some(match entry.response.content.decoded_body() {
                DecodedBody::None => Vec::new(),
                DecodedBody::Text(t) | DecodedBody::Invalid(t) => t.as_bytes().to_vec(),
                DecodedBody::DecodedText(s) => s.into_bytes(),
                DecodedBody::Binary(b) => b,
            });
            return Ok(false);
        }
        Ok(true)
    })?;
    let bytes = match bytes {
        None => {
            return Err(HarError::IndexOutOfRange {
                index: target,
                total,
            })
        }
        Some(b) => b,
    };
    if bytes.is_empty() {
        return Err(HarError::Usage(format!(
            "entry {target} has no response body"
        )));
    }
    std::fs::write(out, &bytes)?;
    eprintln!("wrote {} bytes to {}", bytes.len(), out.display());
    Ok(())
}
