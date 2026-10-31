use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = "Collect float-only metrics")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Start the server in the foreground
    Server,
    /// Record a single metric value
    Record {
        /// Metric name
        name: String,
        /// Metric value
        value: f64,
        /// Unix timestamp in seconds (defaults to now)
        timestamp: Option<i64>,
    },
}

fn main() {
    match Cli::parse().command {
        Command::Server => todo!("start server"),
        Command::Record {
            name,
            value,
            timestamp,
        } => todo!("record {name}={value} at {timestamp:?}"),
    }
}
