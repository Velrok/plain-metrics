# plain metrics

A metrics server process that allows the collection of float only metrics in the form of:

```rust
(ts: Timestamp, name: String, value: f64)
```

Cli

```bash
metrics server # start the server in the foreground
metrics record [--socket <path>] <name>=<value>[@<unix ts in seconds>]... # send one or more metrics to the server
```
