# Preliminary performance tests

Rough numbers, not a rigorous benchmark. Taken 2026-11-05 on a 10-core Mac (macOS), `cargo build --release`.

## Setup (both tests)

- 1 `metrics server`, 9 producers (10 cores minus 1), 5 s run.
- Each producer writes its own metric `benchN`, with the value going up by 1 per record.
- Producers are zsh loops calling `metrics record --socket <sock>`.
- Records counted on the producer side, then checked against the rows in the TSV files.
- The socket lives in `/tmp` because long paths exceed `SUN_LEN`.

## Test 1: one record per call

| Total records | Records/s | Rows on disk |
| ------------: | --------: | -----------: |
|        14,050 |    ~2,800 |       14,050 |

- Each producer made ~1,550 calls, so ~310 calls/s.
- Mostly measures process spawn plus socket connect, not the server.
- The server printed every metric to stdout, which probably cost a little.

## Test 2: batches of 2^P

Producer P sends `2^P` records per call (`name=value` arguments). Server output silenced.

|   P | Batch |  Calls | Records | Records/s |
| --: | ----: | -----: | ------: | --------: |
|   1 |     2 |  1,389 |   2,778 |       555 |
|   2 |     4 |  1,388 |   5,552 |     1,110 |
|   3 |     8 |  1,370 |  10,960 |     2,192 |
|   4 |    16 |  1,374 |  21,984 |     4,396 |
|   5 |    32 |  1,335 |  42,720 |     8,544 |
|   6 |    64 |  1,318 |  84,352 |    16,870 |
|   7 |   128 |  1,239 | 158,592 |    31,718 |
|   8 |   256 |  1,134 | 290,304 |    58,060 |
|   9 |   512 |    876 | 448,512 |    89,702 |
| all |       | ~1,200 | 1.07 M  | ~213,000  |

- All 1,065,754 records reached disk.

## Findings

- Cost is per call, not per record: records/s roughly doubles with each doubling of batch size.
- Batching alone gave ~76× the throughput of test 1.
- From P=7 upwards calls per producer drop (1,389 → 876). The zsh loop building the arguments and the server handling bigger payloads both contribute, so 90k records/s at P=9 is probably a floor.

## Caveats

- Producers and server share the 10 cores.
- The two tests are not directly comparable (server output on in test 1, off in test 2).
- Other things seen: a socket path over `SUN_LEN` and a missing `--data-dir` both panic instead of giving a clean error.

## Test 3: latency of one call (idle machine)

`hyperfine -N --warmup 50 --runs 500`, release build, server running and idle.

| Call        |    Mean |     σ | Min … max      |
| ----------- | ------: | ----: | -------------- |
| 1 record    | 1.4 ms  | 0.0   | 1.3 … 1.6 ms   |
| 512 records | 1.8 ms  | 0.0   | 1.7 … 2.0 ms   |

- Fixed cost ~1.4 ms per call (spawn, parse, connect, send).
- Marginal cost ~0.8 µs per extra record.
- Under load (tests 1 and 2, 10 processes on 10 cores) the same call took ~3.2 ms and ~4 µs per extra record.

### Time to disk

- `record` returns once the data is sent. It does not wait for the write.
- The writer thread drains the channel, writes, then sleeps 1 s (`src/tsv_writer.rs`), so a record reaches the TSV file 0–1 s later (~0.5 s on average).
- A probe measured 259–616 ms. Its timing was phase-locked to the writer's 1 s cycle, so it shows no real spread.
- Whether the client waits for an acknowledgement from the server has not been checked.
