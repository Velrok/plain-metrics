# plain metrics

A metrics server process that allows the collection of float only metrics in the form of:

```rust
(ts: Timestamp, name: String, value: f64)
```

Cli

```bash
metrics server # start the server in the foreground
metrics record <name> <value> [<unix ts>]
```
