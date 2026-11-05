mod actions;
mod cli;
mod metrics;
mod socket_reader;
mod tsv_writer;

use clap::Parser;
use cli::{Cli, Command};

fn main() {
    match Cli::parse().command {
        Command::Server {
            data_dir,
            socket,
            http_port,
        } => {
            let (sender, receiver) = std::sync::mpsc::channel::<metrics::Metric>();
            actions::run_server(&data_dir, socket.as_deref(), http_port, sender, receiver);
        }
        Command::Record { socket, metrics } => {
            let metrics: Vec<metrics::Metric> = metrics.into_iter().map(Into::into).collect();
            actions::record_metrics(&metrics, socket.as_deref());
        }
    }
}
