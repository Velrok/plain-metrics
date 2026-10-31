# CLI help

## `metrics`

```text
$ metrics --help
Collect float-only metrics

Usage: metrics <COMMAND>

Commands:
  server  Start the server in the foreground
  record  Record a single metric value
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## `metrics server`

```text
$ metrics server --help
Start the server in the foreground

Usage: metrics server [OPTIONS] --data-dir <DATA_DIR>

Options:
      --data-dir <DATA_DIR>    Directory the TSV data files are written to (in sub folders)
      --socket <SOCKET>        Path of the local socket `record` connects to
      --http-port <HTTP_PORT>  Port for the HTTP POST listener (no web server without it)
  -h, --help                   Print help
```

## `metrics record`

```text
$ metrics record --help
Record a single metric value

Usage: metrics record <NAME> <VALUE> [TIMESTAMP]

Arguments:
  <NAME>       Metric name
  <VALUE>      Metric value
  [TIMESTAMP]  Unix timestamp in nanoseconds (defaults to now)

Options:
  -h, --help  Print help
```
