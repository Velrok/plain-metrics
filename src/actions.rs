use crate::metrics;
use crate::socket_reader;
use crate::tsv_writer;
use std::io::Write;
use std::os::unix::net::UnixStream;
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

/// Sends the metrics to the running server over its local socket.
pub fn record_metrics(metrics: &[metrics::Metric], socket: Option<&Path>) {
    let socket = socket.map_or_else(socket_reader::default_socket, Path::to_path_buf);
    let result = UnixStream::connect(&socket)
        .map_err(|err| format!("failed to connect to {}: {err}", socket.display()))
        .and_then(|stream| write_metrics(stream, metrics).map_err(|err| err.to_string()));
    match result {
        Ok(()) => println!("sent {} metrics", metrics.len()),
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}

/// Writes one TSV line per metric, the format the socket reader parses.
fn write_metrics(mut out: impl Write, metrics: &[metrics::Metric]) -> std::io::Result<()> {
    for metric in metrics {
        writeln!(out, "{}", metric.to_tsv())?;
    }
    out.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_metrics_writes_one_tsv_line_per_metric() {
        let metrics = [
            metrics::Metric::new("cpu".into(), 0.5, Some(10)),
            metrics::Metric::new("mem".into(), 2.0, Some(20)),
        ];
        let mut out = Vec::new();

        write_metrics(&mut out, &metrics).expect("writing to a Vec cannot fail");

        assert_eq!(
            String::from_utf8(out).expect("utf8"),
            "cpu\t0.5\t10\nmem\t2\t20\n"
        );
    }
}
