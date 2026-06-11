use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::error::Result;

/// Open the HAR input. A path of `-` reads from stdin.
pub fn open(path: &Path) -> Result<Box<dyn BufRead>> {
    if path == Path::new("-") {
        Ok(Box::new(BufReader::new(std::io::stdin())))
    } else {
        Ok(Box::new(BufReader::new(File::open(path)?)))
    }
}
