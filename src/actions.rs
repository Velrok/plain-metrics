use std::path::Path;

pub fn run_server(data_dir: &Path, socket: Option<&Path>, http_port: Option<u16>) {
    todo!("start server in {data_dir:?} on {socket:?}, http port {http_port:?}")
}

pub fn record_metric(name: &str, value: f64, timestamp: Option<i64>) {
    todo!("record {name}={value} at {timestamp:?}")
}
