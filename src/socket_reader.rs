use std::{
    fs,
    io::{BufRead, BufReader},
    os::unix::net::UnixListener,
    path,
    sync::mpsc,
    thread,
};

use crate::metrics;

pub fn run(socket: path::PathBuf, sender: mpsc::Sender<metrics::Metric>) {
    let socket_display = socket.display();

    if socket.exists() {
        eprintln!("Found old socket at {socket_display} -> DELETED");
        fs::remove_file(socket.clone()).expect("Failed to delete existing socket");
    }

    println!("starting listender on {socket_display}");
    let listener = UnixListener::bind(socket).expect("Failed to create socket listener.");
    loop {
        let stream = match listener.accept() {
            Ok((stream, _addr)) => stream,
            Err(msg) => {
                eprintln!("fail to accept listener: {msg}");
                continue;
            }
        };
        let sender = sender.clone();
        thread::spawn(move || {
            for line_result in BufReader::new(stream).lines() {
                match line_result {
                    Ok(line) => match parse_line(&line) {
                        Ok(metric) => {
                            if let Err(msg) = sender.send(metric) {
                                eprintln!("Failed process metrics: {msg}");
                            }
                        }
                        Err(parse_err) => {
                            eprintln!("Failed to parse line: {parse_err}");
                        }
                    },

                    Err(_err) => {
                        eprintln!("Failed to read line");
                    }
                };
            }
        });
    }
}

fn parse_line(line: &str) -> Result<metrics::Metric, String> {
    let mut iter = line.split('\t');
    let name = iter.next().ok_or("missing name")?;
    let value: metrics::Value = match iter.next().map(str::parse).transpose() {
        Ok(val) => val.expect("parsing suceeeded "),
        Err(_) => return Err(String::from("failed to parse value")),
    };

    let at: Option<metrics::TimestampNS> = match iter.next().map(str::parse).transpose() {
        Ok(at) => at,
        Err(_) => return Err(String::from("failed to parse timestamp")),
    };
    Ok(metrics::Metric::new(name.to_owned(), value, at))
}

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
