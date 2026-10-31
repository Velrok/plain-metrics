use crate::metrics;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = "Collect float-only metrics")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Start the server in the foreground
    Server {
        /// Directory the TSV data files are written to (in sub folders)
        #[arg(long)]
        data_dir: PathBuf,
        /// Path of the local socket `record` connects to
        #[arg(long)]
        socket: Option<PathBuf>,
        /// Port for the HTTP POST listener (no web server without it)
        #[arg(long)]
        http_port: Option<u16>,
    },
    /// Record a single metric value
    Record {
        /// Metric name
        name: String,
        /// Metric value
        value: f64,
        /// Unix timestamp in nanoseconds (defaults to now)
        timestamp: Option<metrics::TimestampNS>,
    },
}
