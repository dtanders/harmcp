pub mod json;
pub mod table;
pub mod tsv;

use crate::cli::{Column, OutputFormat};
use crate::har::types::Entry;

#[derive(Clone, Copy)]
pub enum DetailSection {
    Headers,
    Cookies,
    Info,
    Body,
    Timings,
    Stack,
    Ws,
    All,
}

pub fn print_list_header(format: &OutputFormat, columns: &[Column]) {
    match format {
        OutputFormat::Table => table::print_list_header(columns),
        OutputFormat::Tsv => println!("{}", tsv::list_header(columns)),
        OutputFormat::Json => {}
    }
}

pub fn print_list_row(format: &OutputFormat, entry: &Entry, index: usize, columns: &[Column]) {
    match format {
        OutputFormat::Table => table::print_list_row(entry, index, columns),
        OutputFormat::Tsv => println!("{}", tsv::list_row(entry, index, columns)),
        OutputFormat::Json => println!("{}", json::list_row_value(entry, index, columns)),
    }
}

pub fn print_detail(format: &OutputFormat, entry: &Entry, index: usize, section: DetailSection) {
    match format {
        OutputFormat::Table => match section {
            DetailSection::Headers => table::print_detail_headers(entry),
            DetailSection::Cookies => table::print_detail_cookies(entry),
            DetailSection::Info => table::print_detail_info(entry),
            DetailSection::Body => table::print_detail_body(entry),
            DetailSection::Timings => table::print_detail_timings(entry),
            DetailSection::Stack => table::print_detail_stack(entry),
            DetailSection::Ws => table::print_detail_ws(entry),
            DetailSection::All => table::print_detail_all(entry, index),
        },
        OutputFormat::Tsv | OutputFormat::Json => {
            let value = match section {
                DetailSection::Headers => json::headers_value(entry),
                DetailSection::Cookies => json::cookies_value(entry),
                DetailSection::Info => json::info_value(entry),
                DetailSection::Body => json::body_value(entry),
                DetailSection::Timings => json::timings_value(entry),
                DetailSection::Stack => json::stack_value(entry),
                DetailSection::Ws => json::ws_value(entry),
                DetailSection::All => json::all_value(entry, index),
            };
            println!("{}", value);
        }
    }
}
