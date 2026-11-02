use std::{path, sync::mpsc};

use crate::metrics;

pub fn run(socket: path::PathBuf, sender: mpsc::Sender<metrics::Metric>) {}

pub fn default_socket() -> path::PathBuf {
    let dir = [
        std::env::var("XDG_RUNTIME_DIR"),
        std::env::var("TMPDIR"),
        Ok("/tmp".to_owned()),
    ]
    .into_iter()
    .flatten()
    .next()
    .expect("If all fails we fallback to /tmp");
    path::PathBuf::from(dir).join("plain-metrics.sock")
}
