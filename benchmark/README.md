# antixt benchmarks

## v0.4 storefront and request data

```sh
node benchmark/antixt-storefront.mjs
node benchmark/antixt-storefront.mjs --verify
```

This disposable storefront registers a typed catalog and lifecycle observer,
shares one memoized product query between its page and root layout, renders 12
product cards and a dynamic product route, verifies the output, then performs a
2,000-request mixed-route load test. The `--verify` mode runs the correctness
checks used by CI without recording machine-sensitive performance results.

| Measurement | v0.4 storefront |
|---|---:|
| Check / first release build | 383 / 899 ms |
| No-change release build | 37 ms |
| Median render process | 1.60 ms |
| Rendered HTML | 2,846 B |
| Server binary | 622,416 B |
| JavaScript on ordinary routes | 0 B |
| Throughput, concurrency 50 | 16,959.85 req/s |
| p50 / p95 / p99 | 2.66 / 4.43 / 5.70 ms |

The rendered document asserts `data-catalog-loads="1"`, proving that the page
and layout reused the same request-local value rather than querying the shared
catalog twice.

## Canonical application

```sh
node benchmark/antixt-rust.mjs
```

This builds and validates the seven-route v0.4 application, including dynamic
and catch-all parameters, async query handling, streaming, nested layouts,
escaped HTML, optional client injection, and a zero-JavaScript route.

| Measurement | v0.4 | v0.2 |
|---|---:|---:|
| Cold app build | 1,155 ms | 478 ms |
| No-change build | 40 ms | 145 ms |
| One-file app edit | 267 ms | 146 ms |
| Render process | 1.67 ms | 1.72 ms |
| Server binary | 605,600 B | 467,792 B |
| Optional inline runtime | 2,941 B | n/a |
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
| Route scan | 273.73 ms |
| Cold / warm check | 456 / 60 ms |
| Cold / no-change build | 1,628 / 60 ms |
| Shared leaf edit build | 973 ms |
| Throughput | 16,451.97 req/s |
| p50 / p95 / p99 | 2.14 / 6.76 / 13.02 ms |
| RSS after load | 2,801,664 B |

Results are local synthetic samples and are sensitive to machine state, cache
warmth, compiler version, and fixture shape. The native server currently uses a
thread per accepted connection and is not a production HTTP implementation.

- `antixt-rust-results.json`: current seven-route result.
- `antixt-storefront-results.json`: v0.4 typed-state and request-cache result.
- `antixt-scale-results.json`: current 1,000-route/load result.
- `antixt-rust-v02-results.json`: pure-Rust v0.2 baseline.
- `antixt-v01-results.json`: custom-template v0.1 baseline.
- `results.json`, `tooling-results.json`, and `framework-results.json`:
  historical language/toolchain research.
