//! CLI-level tests: run the real `metrics` binary, server and `record` alike.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

const BIN: &str = env!("CARGO_BIN_EXE_metrics");
const TIMEOUT: Duration = Duration::from_secs(10);

/// A scratch directory that is removed again when the test ends.
struct TempDir(PathBuf);

impl TempDir {
    fn new(test: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("plain-metrics-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        Self(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A running `metrics server`, killed when dropped.
struct Server {
    child: Child,
    socket: PathBuf,
    data_dir: PathBuf,
}

impl Server {
    fn start(dir: &TempDir) -> Self {
        let socket = dir.path().join("test.sock");
        let data_dir = dir.path().join("data");
        fs::create_dir_all(&data_dir).expect("create data dir");
        let child = Command::new(BIN)
            .args(["server", "--data-dir"])
            .arg(&data_dir)
            .arg("--socket")
            .arg(&socket)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("start server");
        let server = Self {
            child,
            socket,
            data_dir,
        };
        wait_until(|| server.socket.exists(), "server socket to appear");
        server
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn record(socket: &Path, records: &[&str]) -> Output {
    Command::new(BIN)
        .arg("record")
        .arg("--socket")
        .arg(socket)
        .args(records)
        .output()
        .expect("run record")
}

fn wait_until(mut done: impl FnMut() -> bool, what: &str) {
    let start = Instant::now();
    while !done() {
        assert!(start.elapsed() < TIMEOUT, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Every line of every `.tsv` file below `dir`, sorted.
fn tsv_lines(dir: &Path) -> Vec<String> {
    let mut lines = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(path) = pending.pop() {
        for entry in fs::read_dir(&path).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|ext| ext == "tsv") {
                let content = fs::read_to_string(&path).unwrap_or_default();
                lines.extend(content.lines().map(str::to_owned));
            }
        }
    }
    lines.sort();
    lines
}

#[test]
fn record_writes_metrics_to_the_servers_tsv_files() {
    let dir = TempDir::new("record-writes");
    let server = Server::start(&dir);

    let output = record(
        &server.socket,
        &["cpu=0.5@1700000000.5", "mem=2@1700000001"],
    );

    assert!(output.status.success(), "{output:?}");
    assert_eq!(String::from_utf8_lossy(&output.stdout), "sent 2 metrics\n");
    // The writer drains the channel about once a second, so poll for the file.
    wait_until(
        || tsv_lines(&server.data_dir).len() == 2,
        "metrics in TSV files",
    );
    assert_eq!(
        tsv_lines(&server.data_dir),
        [
            "cpu\t0.5\t1700000000500000000",
            "mem\t2\t1700000001000000000"
        ]
    );
}

#[test]
fn record_without_timestamp_defaults_to_now() {
    let dir = TempDir::new("record-now");
    let server = Server::start(&dir);

    let output = record(&server.socket, &["cpu=1"]);

    assert!(output.status.success(), "{output:?}");
    wait_until(
        || tsv_lines(&server.data_dir).len() == 1,
        "metric in TSV file",
    );
    let lines = tsv_lines(&server.data_dir);
    let timestamp: i64 = lines[0]
        .rsplit('\t')
        .next()
        .and_then(|ts| ts.parse().ok())
        .expect("timestamp column");
    assert!(timestamp > 1_700_000_000_000_000_000, "{lines:?}");
}

#[test]
fn record_fails_when_no_server_is_listening() {
    let dir = TempDir::new("no-server");

    let output = record(&dir.path().join("missing.sock"), &["cpu=1"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("failed to connect"),
        "{output:?}"
    );
}

#[test]
fn record_rejects_malformed_arguments() {
    let dir = TempDir::new("bad-arg");

    for bad in ["cpu", "cpu=abc", "cpu=1@abc"] {
        let output = record(&dir.path().join("unused.sock"), &[bad]);

        assert_eq!(output.status.code(), Some(2), "{bad}: {output:?}");
    }
}

#[test]
fn record_requires_at_least_one_metric() {
    let dir = TempDir::new("no-args");

    let output = record(&dir.path().join("unused.sock"), &[]);

    assert_eq!(output.status.code(), Some(2), "{output:?}");
}
