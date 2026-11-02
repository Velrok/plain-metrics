use crate::metrics;
use crate::socket_reader;
use crate::tsv_writer;
use std::path;
use std::{path::Path, sync::mpsc};

pub fn run_server(
    data_dir: &Path,
    socket: Option<&Path>,
    http_port: Option<u16>,
    sender: mpsc::Sender<metrics::Metric>,
    receiver: mpsc::Receiver<metrics::Metric>,
) {
    if let Some(_) = http_port {
        todo!("Http transport not implemented.")
    }
    // Socket Reader
    // T1_a:
    // listen to socket
    // parse metric
    // send metrics into channel
    // handle errors
    let socket: path::PathBuf = match socket {
        Some(p) => path::PathBuf::from(p),
        None => socket_reader::default_socket(),
    };
    let socker_reader_handle = std::thread::spawn(move || socket_reader::run(socket, sender));

    // T2 (writer)
    // once a sec drain and write TSV to todays file
    let data_dir = data_dir.to_path_buf();
    let tsv_writer_handle = std::thread::spawn(move || tsv_writer::run(data_dir, receiver));

    // ☑️TODO: join them together and cancel the other if one of them fails
    socker_reader_handle
        .join()
        .expect("Expected socker reader to exit without error.");
    tsv_writer_handle
        .join()
        .expect("Expected TSV writer to exit without error.");
}

pub fn record_metric(
    name: &str,
    value: f64,
    timestamp: Option<i64>,
    sender: mpsc::Sender<metrics::Metric>,
) {
    todo!("record {name}={value} at {timestamp:?}")
}
