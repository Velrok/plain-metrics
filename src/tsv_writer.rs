use std::{path, sync::mpsc};

use crate::metrics;

pub fn run(data_dir: path::PathBuf, receiver: mpsc::Receiver<metrics::Metric>) {}
