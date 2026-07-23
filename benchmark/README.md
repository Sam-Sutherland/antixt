# antixt benchmarks

## Canonical application

```sh
node benchmark/antixt-rust.mjs
```

This builds and validates the seven-route v0.3 application, including dynamic
and catch-all parameters, async query handling, streaming, nested layouts,
escaped HTML, optional client injection, and a zero-JavaScript route.

| Measurement | v0.3 | v0.2 |
|---|---:|---:|
| Cold app build | 861 ms | 478 ms |
| No-change build | 37 ms | 145 ms |
| One-file app edit | 231 ms | 146 ms |
| Render process | 1.76 ms | 1.72 ms |
| Server binary | 558,304 B | 467,792 B |
| Optional inline runtime | 2,447 B | n/a |
| Ordinary route JavaScript | 0 B | 0 B |

## Scale and HTTP load

```sh
node benchmark/antixt-scale.mjs
```

The scale benchmark creates a gitignored 1,000-route Cargo app, checks and
builds it, edits a shared leaf component, starts the native server, sends 2,000
requests at concurrency 50, samples RSS, records results, and destroys the app.

| Measurement | Result |
|---|---:|
| Route scan | 136.66 ms |
| Cold / warm check | 448 / 61 ms |
| Cold / no-change build | 1,968 / 62 ms |
| Shared leaf edit build | 1,048 ms |
| Throughput | 17,319.68 req/s |
| p50 / p95 / p99 | 2.40 / 4.41 / 8.90 ms |
| RSS after load | 2,850,816 B |

Results are local synthetic samples and are sensitive to machine state, cache
warmth, compiler version, and fixture shape. The native server currently uses a
thread per accepted connection and is not a production HTTP implementation.

- `antixt-rust-results.json`: current seven-route result.
- `antixt-scale-results.json`: current 1,000-route/load result.
- `antixt-rust-v02-results.json`: pure-Rust v0.2 baseline.
- `antixt-v01-results.json`: custom-template v0.1 baseline.
- `results.json`, `tooling-results.json`, and `framework-results.json`:
  historical language/toolchain research.
