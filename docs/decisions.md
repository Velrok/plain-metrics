# plain-metrics – decision log

Goal: float-only metrics server plus `metrics` CLI (server, record).

## 2026-10-31

- D1 – Binary named `metrics`. README shows `metrics server|record`; package stays `plain-metrics` via [[bin]].
- D2 – clap derive for the CLI. Standard, gives --help/--version for free.
- D3 – "Clippy gem" read as clippy lints. Rust project, gems are Ruby. Enabled clippy::all + pedantic as warnings in Cargo.toml. Pedantic may be dialled back.
- D4 – `record` timestamp is Option<i64> unix seconds. Matches README `[<unix ts>]`. Unit (seconds) is an assumption.
- D5 – Handlers are todo!() stubs. Scaffold only; transport and storage undecided.
- D6 – docs/cli-help.txt holds top-level --help only, not subcommand help.
- D7 – Decision log kept in docs/decisions.txt (user-specified), not the notes vault.
- D8 – Renamed to docs/decisions.md; supersedes the filename in D7.
- D9 – cli-help.txt now holds help for `metrics`, `server` and `record`; supersedes D6.
- D10 – cli-help.txt converted to docs/cli-help.md (headings + fenced blocks); supersedes the filename in D6 and D9.
- D11 – `server` takes optional `--socket <PATH>`: local socket for `record` to talk to it.
- D12 – `server` takes optional `--http-port <PORT>`; HTTP POST listener only starts when given.

- D13 – `record` timestamp is now Option<f64> unix seconds, fractional for sub-second resolution; supersedes the i64 type in D4. f64 gives ~0.2 µs precision at current epoch.

- D14 – `record` timestamp is Option<i64> unix nanoseconds (i64 covers until year 2262); supersedes D13 and the seconds unit in D4.

- D15 – README updated: `record` timestamp is documented as unix nanoseconds, matching D14.

- D16 – `server` requires `--data-dir <PATH>`: metrics are written as TSV files in sub folders of it. Required (no default) until a sensible default is chosen.

## Open questions

- TSV layout: how are sub folders split (per metric name? per day?) and what are the columns/file naming?
- Default socket path when `--socket` is omitted? Does `record` also take `--socket`?
- HTTP POST: request body format (JSON? line protocol?) and bind address (localhost only?)
- Does "default to now" happen in the CLI or on the server?
