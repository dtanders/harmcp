use std::path::Path;

use crate::cli::{Column, ListArgs, OutputFormat};
use crate::error::Result;
use crate::filter::Filters;
use crate::har::stream::stream_entries;
use crate::output;

pub fn run(file: &Path, args: &ListArgs, format: &OutputFormat) -> Result<()> {
    let columns = args.columns.clone().unwrap_or_else(Column::defaults);
    let filters = Filters::from_args(args)?;
    let reader = crate::input::open(file)?;
    output::print_list_header(format, &columns);
    stream_entries(reader, |idx, entry| {
        if filters.matches(&entry) {
            output::print_list_row(format, &entry, idx, &columns);
        }
        Ok(true)
    })?;
    Ok(())
}
