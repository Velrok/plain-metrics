mod actions;
mod cli;
mod metrics;

use clap::Parser;
use cli::{Cli, Command};

fn main() {
    match Cli::parse().command {
        Command::Server {
            data_dir,
            socket,
            http_port,
        } => actions::run_server(&data_dir, socket.as_deref(), http_port),
        Command::Record {
            name,
            value,
            timestamp,
        } => actions::record_metric(&name, value, timestamp),
    }
}
