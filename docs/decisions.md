# plain-metrics – decision log

Goal: float-only metrics server plus `metrics` CLI (server, record).

## 2026-10-31

- D1 – Binary named `metrics`. README shows `metrics server|record`; package stays `plain-metrics` via [[bin]].
- D2 – clap derive for the CLI. Standard, gives --help/--version for free.
- D3 – "Clippy gem" read as clippy lints. Rust project, gems are Ruby. Enabled clippy::all + pedantic as warnings in Cargo.toml. Pedantic may be dialled back.
- D4 – `record` timestamp is Option<i64> unix seconds. Matches README `[<unix ts>]`. Unit (seconds) is an assumption.
- D5 – Handlers are todo!() stubs. Scaffold only; transport and storage undecided.
- D7 – Decision log kept in the repo under `docs/` (user-specified), not the notes vault.
- D11 – `server` takes optional `--socket <PATH>`: local socket for `record` to talk to it.
- D12 – `server` takes optional `--http-port <PORT>`; HTTP POST listener only starts when given.
- D13 – `record` timestamp is now Option<f64> unix seconds, fractional for sub-second resolution; supersedes the i64 type in D4. f64 gives ~0.2 µs precision at current epoch.
- D14 – `record` timestamp is Option<i64> unix nanoseconds (i64 covers until year 2262); supersedes D13 and the seconds unit in D4.
- D16 – `server` requires `--data-dir <PATH>`: metrics are written as TSV files in sub folders of it. Required (no default) until a sensible default is chosen.
- D17 – Default socket is `$XDG_RUNTIME_DIR`, else `$TMPDIR`, else `/tmp`, plus `plain-metrics.sock`. Per-user dir first keeps the socket private. Answers the default-path question; `record --socket` still open.
- D18 – `jiff` chosen over `chrono` to turn ns timestamps into dates. Both add 2 crates (chrono needs `--no-default-features`); jiff has the simpler API and works with defaults. Evidence: `cargo tree` in a scratch project.

- D19 – TSV files are hourly: `<data-dir>/YYYY/MM/DD/YYYY-MM-DDTHHZ.tsv` (UTC), one file per hour, appended to. Was one file per metric timestamp (full ns RFC 3339 name), which made a file per write. No colons in names, so they are safe on any filesystem.

## 2026-11-05

- D20 – `record` takes one or more `name=value[@ts]` arguments (clap `Vec<MetricArg>` via `FromStr`), not a single name/value/timestamp. Several metrics per call; one self-contained token each. Supersedes the single-metric shape from D4/D14. Evidence: `src/cli.rs`, `FromStr` unit tests.
- D21 – The CLI timestamp is fractional unix **seconds** (`f64`), converted to `i64` ns when building a `Metric`. Easier to type and read than ns. `f64` gives ~0.24 µs at the current epoch (as in D13); `f128` is nightly-only. Storage stays ns (D14); supersedes only the ns CLI unit in D14.
- D22 – `record` sends to the running server over its local Unix socket, one TSV line per metric (the format the socket reader parses). Optional `--socket <PATH>`, default as in D17; exits 1 if it cannot connect. Answers the open question on `record --socket`. Evidence: manual run of server + `record` wrote both metrics to hourly TSV files.

## Open questions

- Validate `MetricArg`: reject empty name, NaN/inf and negative timestamps, so the seconds→ns cast cannot saturate or give 0 for NaN.

- TSV layout: how are sub folders split (per metric name? per day?) and what are the columns/file naming?
- HTTP POST: request body format (JSON? line protocol?) and bind address (localhost only?)
- Does "default to now" happen in the CLI or on the server?
