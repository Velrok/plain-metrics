use crate::metrics;
use std::{path::PathBuf, str::FromStr};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = "Collect float-only metrics")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Start the server in the foreground
    Server {
        /// Directory the TSV data files are written to (in sub folders)
        #[arg(long)]
        data_dir: PathBuf,
        /// Path of the local socket `record` connects to
        #[arg(long)]
        socket: Option<PathBuf>,
        /// Port for the HTTP POST listener (no web server without it)
        #[arg(long)]
        http_port: Option<u16>,
    },
    /// Record one or more metrics as `name=value[@seconds]`
    Record {
        /// Path of the server's local socket (defaults to the server's default)
        #[arg(long)]
        socket: Option<PathBuf>,
        /// Records as `name=value[@unix_seconds]` (seconds may be fractional)
        #[arg(required = true, value_name = "NAME=VALUE[@TS]")]
        metrics: Vec<MetricArg>,
    },
}

// ☑️TODO: this is wasteful :/
// having to build an MetricArg just for clap. I dont want to overload my Metric FromStr, because
// I'd like that to parse tsv back into metric
#[derive(Clone)]
pub struct MetricArg {
    name: metrics::Name,
    value: metrics::Value,
    timestamp: Option<f64>,
}

impl FromStr for MetricArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, rest) = s.split_once('=').ok_or(
            "Faild to parse metric arg expect at least name=value but could not find = in arg.",
        )?;
        match rest.split_once('@') {
            Some((val_str, ts_str)) => {
                let value = val_str
                    .parse()
                    .map_err(|e: std::num::ParseFloatError| e.to_string())?;

                let timestamp = ts_str
                    .parse()
                    .map_err(|e: std::num::ParseFloatError| e.to_string())?;

                Ok(MetricArg {
                    name: name.to_owned(),
                    value,
                    timestamp: Some(timestamp),
                })
            }
            None => Ok(MetricArg {
                name: name.to_owned(),
                value: rest
                    .parse()
                    .map_err(|e: std::num::ParseFloatError| e.to_string())?,
                timestamp: None,
            }),
        }
    }
}

impl Into<metrics::Metric> for MetricArg {
    fn into(self) -> metrics::Metric {
        metrics::Metric::new(
            self.name,
            self.value,
            self.timestamp
                .map(|ts| (ts * 1_000_000_000.0) as metrics::TimestampNS),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_name_and_value() {
        let arg: MetricArg = "cpu=0.5".parse().expect("valid arg");

        assert_eq!(arg.name, "cpu");
        assert!((arg.value - 0.5).abs() < f64::EPSILON);
        assert_eq!(arg.timestamp, None);
    }

    #[test]
    fn parses_name_value_and_timestamp() {
        let arg: MetricArg = "cpu=0.5@1700000000".parse().expect("valid arg");

        assert_eq!(arg.name, "cpu");
        assert!((arg.value - 0.5).abs() < f64::EPSILON);
        assert_eq!(arg.timestamp, Some(1_700_000_000.0));
    }

    #[test]
    fn parses_negative_and_exponent_values() {
        let negative: MetricArg = "temp=-5".parse().expect("valid arg");
        let exponent: MetricArg = "tiny=1e-3".parse().expect("valid arg");

        assert!((negative.value + 5.0).abs() < f64::EPSILON);
        assert!((exponent.value - 0.001).abs() < f64::EPSILON);
    }

    #[test]
    fn rejects_missing_equals() {
        assert!("cpu".parse::<MetricArg>().is_err());
    }

    #[test]
    fn rejects_non_numeric_value() {
        assert!("cpu=abc".parse::<MetricArg>().is_err());
        assert!("cpu=abc@1".parse::<MetricArg>().is_err());
    }

    #[test]
    fn rejects_empty_value() {
        assert!("cpu=".parse::<MetricArg>().is_err());
        assert!("cpu=@1".parse::<MetricArg>().is_err());
    }

    #[test]
    fn rejects_bad_timestamp() {
        assert!("cpu=1@".parse::<MetricArg>().is_err());
        assert!("cpu=1@abc".parse::<MetricArg>().is_err());
    }
}
