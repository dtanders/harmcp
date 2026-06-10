use crate::cli::Column;
use crate::har::types::Entry;

pub fn print_list_header(_columns: &[Column]) {}
pub fn print_list_row(_entry: &Entry, _index: usize, _columns: &[Column]) {}
pub fn format_list_row(_entry: &Entry, _index: usize, _columns: &[Column]) -> String { String::new() }
pub fn print_detail_headers(_entry: &Entry) {}
pub fn print_detail_body(_entry: &Entry) {}
pub fn print_detail_timings(_entry: &Entry) {}
pub fn print_detail_stack(_entry: &Entry) {}
pub fn print_detail_all(_entry: &Entry, _index: usize) {}
