# A Unix socket server in Rust (std vs tokio)

> 📘 Personal lesson, not project docs. Both examples were compiled and run.

## 🧠 Mental model

- A **Unix domain socket** is a file path; processes on one machine talk through it
- Server: `bind` a path → `accept` connections → read bytes from each
- Each connection is a `UnixStream`, which implements `Read` + `Write`
- No crates needed: `std::os::unix::net`

| Type          | Role                                      |
| ------------- | ----------------------------------------- |
| `UnixListener` | Bound to a path; yields incoming streams |
| `UnixStream`  | One connection; read/write bytes          |
| `BufReader`   | Adds `.lines()` so you read per message   |

## 💻 Option A: std only

```rust
// What: import the names used below (`use` is Rust's import).
// Why: so we can write `UnixListener` instead of `std::os::unix::net::UnixListener`.
use std::fs;
use std::io::{self, BufRead, BufReader}; // `self` also imports the `io` module itself
use std::os::unix::net::{UnixListener, UnixStream}; // Unix-only: won't compile on Windows
use std::path::Path;
use std::thread;

// What: read lines from one client until it disconnects. Takes the stream by value.
// Why: owning the stream means it is closed automatically when this function
// ends, so we never leak a connection. `io::Result<()>` lets I/O errors propagate.
fn handle(stream: UnixStream) -> io::Result<()> {
    // What: wrap the stream in a `BufReader` and iterate over its lines.
    // Why: a socket delivers raw bytes with no message boundaries. `BufReader`
    // buffers them so `.lines()` can split on '\n'. We treat one line as one
    // message. Each item is an `io::Result<String>` because reading can fail
    // midway. The loop ends on EOF (the client disconnected).
    for line in BufReader::new(stream).lines() {
        // What: print the line; `?` returns early if reading it failed.
        // Why: `?` replaces a `match` on every fallible call.
        println!("got: {}", line?);
    }
    // What: report success.
    // Why: reaching here means the client closed cleanly, not with an error.
    Ok(())
}

// What: program entry point; may return an error.
// Why: returning `Result` lets us use `?` in `main`. An `Err` prints the
// error and exits non-zero instead of panicking.
fn main() -> io::Result<()> {
    // What: build a path value for the socket file.
    // Why: `Path::new` only borrows the string. It does no allocation and
    // touches no filesystem yet.
    let path = Path::new("/tmp/lesson.sock");

    // What: delete the socket file if it already exists.
    // Why: `bind` fails with "address in use" if the file exists, and a
    // crashed earlier run leaves it behind.
    if path.exists() {
        fs::remove_file(path)?;
    }

    // What: create the socket file and start listening on it.
    // Why: from here the kernel queues incoming connections until we call
    // accept, so clients that connect early are not lost.
    let listener = UnixListener::bind(path)?;

    // What: accept clients forever, one `accept` call per client.
    // Why: the listener is ONE socket file, but each client gets its OWN
    // private connection. `accept()` blocks until a client connects, then
    // returns that client's `UnixStream`. Two `nc -U` commands take two trips
    // round this loop; one client that stays connected sends many messages
    // over its single stream (see `handle`).
    // (`listener.incoming()` is the same thing as an iterator. We use an
    // explicit `loop` instead, see the note on the `!` type after the loop.)
    loop {
        // What: unwrap the `Result`; on error, log it and skip to the next client.
        // Why: an accept error is usually about ONE connection (e.g. the client
        // gave up before we accepted) or a passing condition (e.g. out of file
        // descriptors). Using `?` here would end `main` and take the whole
        // server down for every other client. `continue` jumps straight to the
        // next trip round the loop instead.
        let stream = match listener.accept() {
            // `accept` also returns the client's address; `_addr` is unused.
            Ok((stream, _addr)) => stream,
            Err(e) => {
                eprintln!("accept error: {e}");
                continue;
            }
        };

        // What: handle this client on a new OS thread.
        // Why: `handle` blocks while reading. Without a thread, client #2
        // would wait in the kernel queue until client #1 disconnected.
        // `move` transfers ownership of `stream` into the closure, which is
        // required because the thread may outlive this loop iteration.
        thread::spawn(move || {
            // What: run the handler and log any error.
            // Why: `?` needs a function that returns a `Result`, and this
            // closure returns `()`. One bad client must not stop the server.
            if let Err(e) = handle(stream) {
                eprintln!("connection error: {e}"); // stderr; `{e}` inlines the variable
            }
        });
    }

    // Nothing to return here, on purpose. A `loop` with no `break` has the
    // "never" type `!`, which coerces to any type, so it satisfies
    // `io::Result<()>` by itself. The compiler PROVES the loop never ends.
    // A `for` over `incoming()` would need a trailing `Ok(())` (the compiler
    // can't know the iterator never yields `None`), or an `unreachable!()`
    // that panics if that assumption is ever wrong.
}
```

## 🔍 Walkthrough

- **`remove_file` first**: `bind` fails with `AddrInUse` if the file exists. A crash leaves a stale file behind
- **`listener.accept()`**: blocks until a client connects; returns `io::Result<(UnixStream, SocketAddr)>`
- **`loop` without `break`**: has type `!` (never), so `main` needs no trailing `Ok(())`. `listener.incoming()` is the iterator form of the same call
- **`thread::spawn(move || …)`**: one thread per connection so a slow client can't block others. `move` hands the stream to the thread
- **`BufReader::lines()`**: a message = one `\n`-terminated line. It returns `Ok` lines until the client closes (EOF)
- **Errors in the thread**: can't use `?` in the closure, so log and drop the connection

## ▶️ Try it

```sh
rustc srv.rs && ./srv
# other terminal
printf 'cpu.load 0.42\n' | nc -U /tmp/lesson.sock
```

## 💻 Option B: tokio (async)

```toml
# Cargo.toml
tokio = { version = "1", features = ["fs", "rt-multi-thread", "macros", "net", "io-util", "signal"] }
```

```rust
use std::io;
use std::path::Path;
use tokio::fs; // async versions of the std fs functions
use tokio::io::{AsyncBufReadExt, BufReader}; // async `BufReader`; the trait provides `.lines()`
use tokio::net::{UnixListener, UnixStream}; // same names as std, but `accept()` is async

// What: read lines from one client until it disconnects.
// Why: `async fn` returns a "future", a paused computation that only runs
// when something `.await`s it. Calling it does nothing by itself, which lets
// the runtime interleave many of these on few threads.
async fn handle(stream: UnixStream) -> io::Result<()> {
    // What: wrap the stream in a buffered reader and get its line reader.
    // Why: same reason as std (sockets are byte streams, we want lines). But
    // this is not an iterator: we pull one line at a time with `next_line()`.
    let mut lines = BufReader::new(stream).lines();

    // What: fetch lines until the client disconnects.
    // Why: `.await` pauses THIS task until data arrives and lets the runtime
    // run other tasks meanwhile, which is how one thread serves many clients.
    // The result is `Ok(Some(line))`, `Ok(None)` on EOF, or `Err`. `?` handles
    // the `Err`, and `while let Some(..)` stops on EOF.
    while let Some(line) = lines.next_line().await? {
        println!("got: {line}");
    }
    Ok(())
}

// What: program entry point that runs on tokio's runtime.
// Why: Rust has no built-in async runtime. `#[tokio::main]` rewrites `main`
// into a normal fn that builds a runtime (thread pool + scheduler) and runs
// this body on it.
#[tokio::main]
async fn main() -> io::Result<()> {
    let path = Path::new("/tmp/lesson-tokio.sock");

    // What: delete any stale socket file and ignore the result.
    // Why: `bind` fails if the file exists. The file usually doesn't, and that
    // "error" is fine, so `let _ =` discards it.
    let _ = fs::remove_file(path).await;

    // What: create the socket file and start listening.
    // Why: binding is synchronous. It is fast and waits for no client.
    let listener = UnixListener::bind(path)?;

    loop {
        // What: wait for either a new client or Ctrl-C, whichever comes first.
        // Why: `select!` runs several futures at once and takes the FIRST to
        // finish, dropping (cancelling) the others. That lets us stop
        // accepting and clean up on Ctrl-C.
        tokio::select! {
            // Branch 1: `pattern = future => body`
            accepted = listener.accept() => {
                // What: take the new client's connection and peer address.
                // Why: each `accept` resolves to ONE client with a private
                // stream. The address is ignored. The next client is handled
                // on the next loop iteration.
                // On error, log and `continue` rather than `?`: a failed accept
                // is usually one connection's problem, not a reason to stop
                // the server (see the std example).
                let (stream, _) = match accepted {
                    Ok(pair) => pair,
                    Err(e) => {
                        eprintln!("accept error: {e}");
                        continue;
                    }
                };

                // What: handle this client in a new tokio task.
                // Why: a task needs a few hundred bytes of state, not a whole
                // OS thread stack, so thousands are fine. `async move` is an
                // async block that takes ownership of `stream`, like
                // `move ||` in the std version.
                tokio::spawn(async move {
                    if let Err(e) = handle(stream).await {
                        eprintln!("connection error: {e}");
                    }
                });
            }
            // Branch 2: Ctrl-C arrived, so leave the loop.
            _ = tokio::signal::ctrl_c() => break,
        }
    }

    // What: remove the socket file, then exit cleanly.
    // Why: the OS does not delete it for us. The std version leaves it behind.
    fs::remove_file(path).await?;
    Ok(())
}
```

- Same shape: bind → accept → spawn a task per connection → read lines
- `tokio::spawn` creates a cheap **task**, not an OS thread
- `select!` races `accept` against `Ctrl-C`, so the socket file is **cleaned up on exit**
- `lines()` is a `while let` loop with `.await`, not an iterator
- Forgetting a feature flag gives a confusing "unresolved import" (needed `fs` here)

## ⚖️ std vs tokio

| Aspect            | std                          | tokio                              |
| ----------------- | ---------------------------- | ---------------------------------- |
| Dependencies      | 0                            | ~20 crates in the tree             |
| Release binary    | ~0.5 MB                      | ~0.9 MB                            |
| Concurrency       | OS thread per connection     | Task per connection                |
| Scales to         | Hundreds of clients          | Tens of thousands                  |
| Graceful shutdown | Manual (signal handling hard)| `select!` + `ctrl_c()` in 1 line   |
| Mixing with HTTP  | Awkward                      | Native (axum, hyper)               |
| Learning curve    | Low                          | `async`, `.await`, feature flags   |
| Compile time      | Instant                      | Noticeably longer                  |

## 🧭 Which to pick

- **std**: few local clients, one-off tools, want zero dependencies
- **tokio**: already need async (e.g. an HTTP POST listener alongside the socket), many clients, or clean shutdown
- For `plain-metrics` the planned `--http-port` listener tips it towards **tokio**: one runtime serves both
- Rule of thumb: don't pull in async for a single local socket; do once a second listener appears

## ⚠️ Gotchas

- `continue` on accept errors can **busy-loop** if the error persists (e.g. out of file descriptors: accept fails instantly, forever). A real server sleeps briefly (`thread::sleep` / `tokio::time::sleep`) after an error

- The socket file is **not** removed on exit; clean up on shutdown or at next start
- Path length limit is ~104 bytes on macOS (108 on Linux)
- Framing is your job: a socket is a byte stream, not messages. Lines are the simplest framing
- Thread-per-connection is fine for std; see the comparison above for when to switch

## ➡️ Next steps

- Parse each line into a `Metric` (`name value [ts]`)
- Replace `println!` with writing to the TSV files
- Handle `Ctrl-C` to remove the socket file
