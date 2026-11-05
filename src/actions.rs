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
    let tsv_writer_handle = std::thread::spawn(move || tsv_writer::run(&data_dir, receiver));

    // ☑️TODO: join them together and cancel the other if one of them fails
    socker_reader_handle
        .join()
        .expect("Expected socker reader to exit without error.");
    tsv_writer_handle
        .join()
        .expect("Expected TSV writer to exit without error.");
}

pub fn record_metrics(metrics: Vec<metrics::Metric>, sender: &mpsc::Sender<metrics::Metric>) {
    let mut any_errors = false;
    let mut send_successfully = 0;
    for metric in metrics {
        match sender.send(metric) {
            Ok(()) => send_successfully += 1,
            Err(err) => {
                // eprint an err with the metrics that failed to send
                eprintln!("failed to send metric: {:?}", err.0);
                any_errors = true;
            }
        }
    }
    println!("sent {send_successfully} metrics successfully");
    if any_errors {
        std::process::exit(1);
    }
}
