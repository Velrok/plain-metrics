use core::time;
use std::io::Write;
use std::{collections::HashMap, fs, io, path, sync::mpsc};

use crate::metrics;

pub fn run(data_dir: &path::PathBuf, receiver: mpsc::Receiver<metrics::Metric>) {
    // err unless data exists; is a dir ; and writable
    if !data_dir.is_dir() {
        panic!(
            "data dir {} is missing or not a directory",
            data_dir.display()
        );
    }
    let readonly = match data_dir.metadata() {
        Ok(meta) => meta.permissions().readonly(),
        Err(err) => panic!("cannot read data dir {}: {}", data_dir.display(), err),
    };
    if readonly {
        panic!("data dir {} is not writable", data_dir.display());
    }

    loop {
        match receiver.try_recv() {
            Ok(metric) => match batch_write(&data_dir, metric, receiver.try_iter().collect()) {
                Ok(()) => {}
                Err(err) => eprintln!("failed to write TSV batch: {err}"),
            },
            Err(err) => match err {
                mpsc::TryRecvError::Empty => {}
                mpsc::TryRecvError::Disconnected => {
                    eprintln!("TSV channel disconnected, exiting thread!");
                    return;
                }
            },
        }
        std::thread::sleep(time::Duration::from_secs(1));
    }
}

fn batch_write(
    data_dir: &path::PathBuf,
    metric: metrics::Metric,
    messages: Vec<metrics::Metric>,
) -> anyhow::Result<()> {
    let mut batch: HashMap<path::PathBuf, Vec<metrics::Metric>> = HashMap::new();

    batch
        .entry(path_for_metric(data_dir, &metric)?)
        .or_default()
        .push(metric);

    for metric in messages {
        batch
            .entry(path_for_metric(data_dir, &metric)?)
            .or_default()
            .push(metric);
    }

    for (path, metrics) in batch {
        println!("Adding metrics to {path:?}");
        create_parent_dir(&path)?;
        let mut out = io::BufWriter::new(append_only_writer(&path)?);
        for metric in metrics {
            let line = metric.to_tsv();
            writeln!(out, "{line}")?;
        }
        out.flush()?;
    }

    Ok(())
}

fn append_only_writer(path: &path::PathBuf) -> io::Result<fs::File> {
    fs::OpenOptions::new().create(true).append(true).open(path)
}

fn create_parent_dir(path: &path::Path) -> anyhow::Result<()> {
    match path.parent() {
        Some(parent_dir) => Ok(fs::create_dir_all(parent_dir)?),
        None => Err(anyhow::anyhow!("no parent dir for {}", path.display())),
    }
}

fn path_for_metric(
    data_dir: &path::Path,
    metric: &metrics::Metric,
) -> anyhow::Result<path::PathBuf> {
    let Ok(ts) = jiff::Timestamp::from_nanosecond(i128::from(metric.at)) else {
        return Err(anyhow::anyhow!(
            "cannot parse at: {} as timestamp",
            metric.at
        ));
    };

    // Rebuilding from components drops repeated and trailing separators (e.g. from `$TMPDIR`).
    let mut data_dir: path::PathBuf = data_dir.components().collect();
    data_dir.push(ts.strftime("%Y/%m/%d/").to_string());
    data_dir.push(ts.strftime("%Y-%m-%dT%HZ.tsv").to_string());
    Ok(data_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_for_metric_uses_date_folders_and_hourly_file_name() {
        // 2026-11-03T12:00:00.123456789Z
        let at: metrics::TimestampNS = 1_793_707_200_123_456_789;
        let metric = metrics::Metric::new("cpu".into(), 1.0, Some(at));
        let data_dir = path::PathBuf::from("/data");

        let path = path_for_metric(&data_dir, &metric)
            .ok()
            .expect("valid timestamp");

        assert_eq!(
            path,
            path::PathBuf::from("/data/2026/11/03/2026-11-03T12Z.tsv")
        );
    }

    #[test]
    fn path_for_metric_has_no_double_slashes_for_data_dir_with_trailing_slash() {
        let at: metrics::TimestampNS = 1_793_707_200_123_456_789;
        let metric = metrics::Metric::new("cpu".into(), 1.0, Some(at));
        let data_dir = path::PathBuf::from("/tmp//data//");

        let path = path_for_metric(&data_dir, &metric)
            .ok()
            .expect("valid timestamp");

        // PathBuf equality ignores repeated separators, so compare the text.
        assert_eq!(
            path.to_string_lossy(),
            "/tmp/data/2026/11/03/2026-11-03T12Z.tsv"
        );
    }
}
