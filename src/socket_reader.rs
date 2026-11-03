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
                        Ok(metric) => match sender.send(metric.clone()) {
                            Ok(_) => println!("Passing metric to TSV writer: {metric:?}"),
                            Err(msg) => {
                                eprintln!("Failed process metrics: {msg}");
                            }
                        },
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
    let name = iter
        .next()
        .ok_or_else(|| format!("missing name in line {line:?}"))?;
    let raw_value = iter.next().ok_or_else(|| {
        format!("missing value in line {line:?} (expected <name>\\t<value>[\\t<timestamp>])")
    })?;
    let value: metrics::Value = raw_value
        .parse()
        .map_err(|err| format!("failed to parse value {raw_value:?} in line {line:?}: {err}"))?;

    let at: Option<metrics::TimestampNS> = iter
        .next()
        .map(|raw_at| {
            raw_at.parse().map_err(|err| {
                format!("failed to parse timestamp {raw_at:?} in line {line:?}: {err}")
            })
        })
        .transpose()?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn now_ns() -> metrics::TimestampNS {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock is after epoch")
            .as_nanos()
            .try_into()
            .expect("fits in i64")
    }

    #[test]
    fn name_and_value_defaults_timestamp_to_now() {
        let before = now_ns();
        let metric = parse_line("hello\t12").expect("valid line");
        let after = now_ns();

        assert_eq!(metric.name, "hello");
        assert!((metric.value - 12.0).abs() < f64::EPSILON);
        assert!((before..=after).contains(&metric.at));
    }

    #[test]
    fn explicit_timestamp_is_kept() {
        let metric = parse_line("hello\t12\t1700000000000000000").expect("valid line");

        assert_eq!(metric.name, "hello");
        assert!((metric.value - 12.0).abs() < f64::EPSILON);
        assert_eq!(metric.at, 1_700_000_000_000_000_000);
    }

    #[test]
    fn value_can_be_fractional_negative_or_scientific() {
        for (input, expected) in [
            ("m\t1.5", 1.5),
            ("m\t-3", -3.0),
            ("m\t0", 0.0),
            ("m\t1e3", 1000.0),
        ] {
            let metric = parse_line(input).expect(input);
            assert!((metric.value - expected).abs() < f64::EPSILON, "{input}");
        }
    }

    #[test]
    fn name_may_contain_dots_and_spaces() {
        let metric = parse_line("cpu.load avg\t0.5").expect("valid line");

        assert_eq!(metric.name, "cpu.load avg");
    }

    #[test]
    fn missing_value_is_an_error() {
        assert_eq!(
            parse_line("hello").err().as_deref(),
            Some("missing value in line \"hello\" (expected <name>\\t<value>[\\t<timestamp>])")
        );
    }

    #[test]
    fn empty_line_is_an_error() {
        assert!(parse_line("").is_err());
    }

    #[test]
    fn non_numeric_value_is_an_error() {
        assert_eq!(
            parse_line("hello\tabc").err().as_deref(),
            Some(
                "failed to parse value \"abc\" in line \"hello\\tabc\": \
                 invalid float literal"
            )
        );
    }

    #[test]
    fn non_numeric_timestamp_is_an_error() {
        assert_eq!(
            parse_line("hello\t1\tyesterday").err().as_deref(),
            Some(
                "failed to parse timestamp \"yesterday\" in line \"hello\\t1\\tyesterday\": \
                 invalid digit found in string"
            )
        );
    }

    #[test]
    fn fractional_timestamp_is_an_error() {
        assert!(parse_line("hello\t1\t1.5").is_err());
    }
}
