# CLI help

## `metrics`

```text
$ metrics --help
Collect float-only metrics

Usage: metrics <COMMAND>

Commands:
  server  Start the server in the foreground
  record  Record one or more metrics as `name=value[@seconds]`
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
Record one or more metrics as `name=value[@seconds]`

Usage: metrics record [OPTIONS] <NAME=VALUE[@TS]>...

Arguments:
  <NAME=VALUE[@TS]>...  Records as `name=value[@unix_seconds]` (seconds may be fractional)

Options:
      --socket <SOCKET>  Path of the server's local socket (defaults to the server's default)
  -h, --help             Print help
```
