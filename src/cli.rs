use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "harmcp", about = "Extract structured data from HAR log files")]
pub struct Cli {
    /// Path to the HAR file
    pub file: PathBuf,
    #[command(subcommand)]
    pub command: Command,
    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Table, global = true)]
    pub format: OutputFormat,
}

#[derive(Subcommand)]
pub enum Command {
    /// List all entries with optional filtering and column selection
    List(ListArgs),
    /// Request and response headers for entry N
    Headers { index: usize },
    /// Request payload and response body for entry N
    Body { index: usize },
    /// Timing breakdown for entry N
    Timings { index: usize },
    /// Initiator call stack for entry N
    Stack { index: usize },
    /// All details for entry N
    All { index: usize },
}

#[derive(Args)]
pub struct ListArgs {
    /// Columns to display, comma-separated: index,method,status,url,mime,size,time
    #[arg(long, value_delimiter = ',')]
    pub columns: Option<Vec<Column>>,
    /// Filter by HTTP method, case-insensitive exact match (e.g. GET, post)
    #[arg(long)]
    pub method: Option<String>,
    /// Filter by status: exact (200), wildcard (4xx, 20x), or range (400-499)
    #[arg(long)]
    pub status: Option<String>,
    /// Filter by URL substring, case-insensitive
    #[arg(long)]
    pub url: Option<String>,
    /// Filter by URL regex (Rust regex syntax)
    #[arg(long = "url-regex")]
    pub url_regex: Option<String>,
    /// Filter by MIME type substring, case-insensitive
    #[arg(long)]
    pub mime: Option<String>,
    /// Minimum response body size in bytes
    #[arg(long)]
    pub min_size: Option<i64>,
    /// Maximum response body size in bytes
    #[arg(long)]
    pub max_size: Option<i64>,
}

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum OutputFormat {
    #[default]
    Table,
    Tsv,
    Json,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum Column {
    Index,
    Method,
    Status,
    Url,
    Mime,
    Size,
    Time,
}

impl Column {
    pub fn defaults() -> Vec<Column> {
        vec![
            Column::Index,
            Column::Method,
            Column::Status,
            Column::Url,
            Column::Mime,
            Column::Size,
            Column::Time,
        ]
    }
}
