# antixt framework research summary

Date consolidated: 23 July 2026

## Objective

The investigation asked whether a Next.js-like framework could preserve clean
file-based routing while improving build speed, startup time, server rendering,
and shipped JavaScript. The prototypes intentionally used standard libraries
where possible so runtime and toolchain costs remained visible.

The resulting product direction is a Rust-owned, server-first framework for
applications authored in ordinary Rust, with zero client JavaScript by default
and optional interactivity as a future layer.

## What we tested

### Equivalent landing page

The first benchmark compiled and rendered the same 3,259-byte landing page with
TypeScript 7, direct Rust, Go, a C-written antixt compiler emitting C, and the
same compiler emitting Rust. Every implementation passed the same SHA-256
output check.

| Implementation | Check | Build | Rebuild | Process render | Artifact |
|---|---:|---:|---:|---:|---:|
| TypeScript 7 + Node | 191.75 ms | 195 ms | 192 ms | 56.95 ms | 501 B plus Node |
| Rust | included in build | 232 ms | 228 ms | 3.63 ms | 468,344 B |
| Go | included in build | 124 ms | 95 ms | 6.22 ms | 7,691,202 B |
| antixt emitting C | 4.76 ms | 51 ms | 50 ms | 1.95 ms | 34,288 B |
| antixt emitting Rust | 1.65 ms | 232 ms | 247 ms | 3.04 ms | 485,440 B |

Source: [`benchmark/results.json`](benchmark/results.json).

Interpretation: native processes eliminated most Node startup cost. C produced
the smallest and fastest tiny static server; Rust remained close at runtime but
paid a larger compile and binary-size cost. This test did not represent a real
module graph or dynamic application.

### Native tooling workloads

Go, Rust, and C implementations processed the same 1,200-file fixture and
10,000-node/24,000-edge graph. Each operation used seven samples and recorded
the median.

| Language | Build | Binary | Startup | Scan | Graph | Invalidate | SSR | Refresh |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Go | 243 ms | 2,583,170 B | 2.15 ms | 21.49 ms | 5.23 ms | 10.91 ms | 12.49 ms | 16.17 ms |
| Rust | 1,059 ms | 537,680 B | 2.15 ms | 20.09 ms | 2.98 ms | 4.97 ms | 9.14 ms | 10.02 ms |
| C | 94 ms | 35,368 B | 1.83 ms | 18.48 ms | 2.69 ms | 5.47 ms | 11.43 ms | 9.78 ms |

Source: [`benchmark/tooling-results.json`](benchmark/tooling-results.json).

Interpretation: C led build, size, startup, scanning, and several tiny
operations. Rust won incremental invalidation and SSR and was competitive on
graph and refresh work. Go was easy to build and initially attractive as the
implementation language behind TypeScript 7, but it did not win a runtime
workload in this fixture.

### Framework-shaped routing prototype

The next prototype added three real method/path routes, root and nested layouts,
typed components, escaped values, and an HTML action. A dependency-free Node
project scanner fed a C compiler that emitted either C or Rust servers.

| Backend | Project scan | Route check | Build | Rebuild | Process render | Binary | Client JS |
|---|---:|---:|---:|---:|---:|---:|---:|
| C | 50.83 ms | 1.74 ms | 101 ms | 103 ms | 1.56 ms | 34,376 B | 0 B |
| Rust | 47.94 ms | 1.28 ms | 265 ms | 303 ms | 2.09 ms | 468,936 B | 0 B |

Both backends produced identical output across `GET /`, `GET /about`, and
`POST /newsletter`.

Source: [`benchmark/framework-results.json`](benchmark/framework-results.json).

Interpretation: zero-JavaScript pages and server actions are realistic, not
just conceptual. However, this pipeline did not prove a Rust-owned framework:
its scanner was JavaScript and its compiler frontend was C. That limitation led
directly to the v0.1 rewrite.

## Other architectural conclusions

### HTMX-style interaction

Returning HTML fragments for mutations is a useful default. It allows forms and
links to remain valid HTML while a future tiny client runtime can progressively
enhance targeted replacement, navigation, and optimistic states. Close to zero
JavaScript is realistic for content, forms, navigation, authentication, and
many dashboards. Rich local state, offline behavior, editors, animation, and
canvas applications will still need client code.

### WebAssembly

Wasm does not automatically make a browser application faster. DOM work still
crosses the JavaScript/Web API boundary, downloads and instantiation have costs,
and a Wasm server does not remove Node unless it runs in a different host. Wasm
is promising for isolated CPU-heavy transforms and portable plugins, not as the
default rendering answer.

### A custom language

A language can be implemented in Rust without being "based on" Rust syntax.
The important choices are grammar, type system, semantics, compiler host, and
target. antixt keeps v0.1 deliberately HTML-shaped so source remains legible to
people and coding agents. A broader language should only grow in response to
validated framework requirements.

## Why Rust won the implementation decision

The benchmarks did not show Rust winning every number. The decision optimizes
for the whole framework lifecycle:

- memory and type safety are valuable in parsers, caches, concurrent servers,
  plugin boundaries, and agent-authored patches;
- Rust provides native startup and strong runtime performance without a garbage
  collector;
- enums, ownership, and exhaustive matching make compiler state easier to
  evolve safely;
- one Rust codebase can own scanning, validation, code generation, the dev
  server, and production runtime;
- unsafe or lower-level optimizations remain possible behind narrow interfaces
  after profiling.

C remains a useful performance floor and research reference. Go remains relevant
to understanding TypeScript 7's native toolchain. Neither is part of the antixt
v0.1 build path.

## Historical antixt v0.1 architecture

```text
app/**/*.antixt + components/**/*.antixt
                  │
                  ▼
        Rust scanner and validator
                  │
                  ▼
       typed deterministic route model
                  │
                  ▼
          Rust server code generator
                  │
                  ▼
              rustc -O
                  │
                  ▼
       standalone native HTTP server
```

This architecture established the native runtime and file-routing proof, but
its custom source parser and templates were removed by the pure-Rust authoring
experiment documented below. The retained v0.1 result remains the comparison
baseline.

## v0.1 verification

After replacing the JavaScript scanner and C compiler with the Rust-owned CLI,
the canonical three-route application produced these local results:

| CLI build | CLI startup | Check | App build | Rebuild | Process render | Server | Client JS |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 850 ms | 4.22 ms | 2.27 ms | 155 ms | 154 ms | 2.38 ms | 487,480 B | 0 B |

The benchmark verifies escaped interpolation, nested layout output, all three
method/path routes, a non-zero exit for a missing render route, and one combined
output SHA-256. The CLI itself was 677,360 bytes in this release build.

Source: [`benchmark/antixt-v01-results.json`](benchmark/antixt-v01-results.json).

### Development reload follow-up

The Rust CLI now includes a persistent `antixt dev` process. It recompiles the
typed route model in memory every 100 ms and injects a development-only reload
client. In the in-app browser test, changing a generated app's heading appeared
without a manual browser refresh inside a 600 ms observation window; the route
compiler itself reported 0.44 ms. A deliberately unknown template value showed
an escaped error overlay while preserving the last valid page, and fixing the
source recovered automatically with a 0.23 ms route compile.

This is automatic full-page hot reload. State-preserving component replacement
and dependency-directed invalidation remain future work. Production builds do
not include the reload client.

## Pure Rust authoring experiment

The next iteration removed the `.antixt` source language entirely. Application
pages, layouts, method handlers, and components became ordinary `.rs` modules.
antixt now scans filenames and generates only `#[path]` module declarations,
typed handler wrappers, and the route table. Cargo and rustc own all parsing and
type checking, which means Rust Analyzer provides the complete language tooling
without an antixt-specific LSP.

The typed HTML builder represents elements as Rust values. Text and attributes
escape by default, layouts accept and return `Html`, components are normal
functions with Rust props, and generated wrappers make invalid page/action
signatures compile errors.

### Performance comparison

| Measurement | Pure Rust v0.2 | Template-language v0.1 |
|---|---:|---:|
| CLI build | 828 ms | 850 ms |
| CLI startup | 1.70 ms | 4.22 ms |
| Route scan | 1.62 ms | n/a |
| Cold Cargo check | 166 ms | n/a |
| Warm Cargo check | 50.28 ms | 2.27 ms template check |
| Cold release build | 478 ms | 155 ms |
| No-change release rebuild | 145 ms | 154 ms |
| One-file release rebuild | 146 ms | n/a |
| Render process | 1.72 ms | 2.38 ms |
| Native server | 467,792 B | 487,480 B |
| Client JavaScript | 0 B | 0 B |

Two edits through the real `antixt dev` supervisor compiled, restarted the child
server, and became reloadable in 104.52 ms and 101.37 ms. This is slower than
the earlier 0.2–0.4 ms static-template swap because application logic now goes
through rustc, but it remains close to a tenth of a second while gaining native
types, refactoring, completion, diagnostics, and navigation.

The cold-build regression is the clear cost: the app's Cargo target compiles the
framework dependency as well as route code. Sharing compiled framework artifacts,
using a workspace target, and measuring larger applications are the next
optimization opportunities. Warm rebuild, runtime startup, and binary size all
improved slightly in this first implementation.

Source: [`benchmark/antixt-rust-v02-results.json`](benchmark/antixt-rust-v02-results.json).

## antixt v0.3 full-stack experiment

v0.3 implemented the six missing framework seams as one integrated release:

1. `[slug]` and terminal `[...path]` directories generate route-specific Rust
   `Params` structs populated with decodable `Value` fields. Static routes are
   ordered before dynamic and catch-all patterns.
2. `Context` exposes query values, URL-encoded forms, headers, cookies, raw
   bodies, and fragment detection. `Response` supports status, headers,
   redirects, full bodies, and stream bodies.
3. `AsyncResponse` accepts normal Rust futures, and the dependency-free executor
   honours wakeups. `Response::stream` writes HTTP chunked transfer incrementally.
4. The declarative `view!` macro provides nested, escaped HTML while remaining
   valid Rust understood by rust-analyzer.
5. The scale benchmark generates and destroys a 1,000-route application, edits
   a shared leaf, and records throughput, latency percentiles, and memory.
6. HTML elements can opt into fragment requests and client islands. JavaScript
   modules under `client/` are embedded in the binary and loaded on demand. The
   2.4 KB runtime is injected only into documents containing enhancement markers.

The in-app browser verified that a counter island mounted and retained local
state, a form POST replaced only `#newsletter-result`, and a generated dynamic
route received `browser-tested` through its typed parameter struct.

### Seven-route application

| Measurement | v0.3 | v0.2 |
|---|---:|---:|
| Cold release build | 861 ms | 478 ms |
| No-change release build | 37 ms | 145 ms |
| One-file application edit | 231 ms | 146 ms |
| Render process | 1.76 ms | 1.72 ms |
| Native server | 558,304 B | 467,792 B |
| Inline enhancement runtime | 2,447 B opt-in | none |
| Ordinary route JavaScript | 0 B | 0 B |

Cold build and binary size increased because v0.3 contains request parsing,
route patterns, futures, streaming, and client serving. The no-change build
improved after generated source became write-on-change. Application edit time
rose by 85 ms in this sample but remained below a quarter second.

Source: [`benchmark/antixt-rust-results.json`](benchmark/antixt-rust-results.json).

### 1,000-route and HTTP load result

| Measurement | Result |
|---|---:|
| Route scan | 136.66 ms |
| Cold / warm check | 448 / 61 ms |
| Cold / no-change release build | 1,968 / 62 ms |
| Shared leaf edit release build | 1,048 ms |
| Server startup probe | 263.32 ms |
| Throughput at concurrency 50 | 17,319.68 req/s |
| p50 / p95 / p99 | 2.40 / 4.41 / 8.90 ms |
| RSS after load | 2,850,816 B |

The 1,000-route result shows that filename scanning and generated wiring remain
manageable. The expensive case is correctly the shared-leaf edit: all 1,000
route modules depend on it. Future invalidation work should distinguish shared
and route-local components and measure debug development builds separately from
release rebuilds.

The HTTP numbers prove the native runtime can serve concurrent local traffic;
they are not production capacity claims. The server uses one OS thread per
connection and has not been tested for slow clients, cancellation, TLS, proxy
semantics, denial-of-service resistance, or long-running workloads.

Source: [`benchmark/antixt-scale-results.json`](benchmark/antixt-scale-results.json).

## Typed utility CSS spike

The first CSS experiment keeps the same rule as Rust authoring: antixt must not
scan or parse application source. Utilities are normal Rust values constructed
from enums such as `Display`, `Space`, `Color`, and `Breakpoint`. `hover`,
`focus_visible`, and `at` wrap the same `Utility` type, so arrays can mix base,
state, and responsive declarations while rustc rejects unknown tokens.

`Html::render` walks the completed node tree, creates deterministic FNV-derived
class names from canonical declarations, removes duplicate rules, and appends a
`data-antixt-css` style block after authored head styles. Non-document fragments
carry their own required rules. No utility catalog or unused stylesheet ships.

The documentation landing page now uses five generated utilities for its action
row and has no authored CSS for that layout. Its production output contained
five atomic rules and five deterministic classes while remaining zero-JavaScript.
The framework test suite increased from 15 to 18 tests.

This proves type-safe utility composition and render-time dead-style elimination.
It does not yet solve application-defined theme types, property conflict
diagnostics, cascade layers, container queries, stylesheet caching, CSP-safe
external extraction, or utility CSS in long-lived streamed documents.

## Adjacent framework research and inspiration

Research into experimental Rust-first, server-rendered frameworks exposed a
useful alternative set of trade-offs. A modular, batteries-included stack can
build on the Tokio, Hyper, Tower, and Axum ecosystem, while antixt is currently
a dependency-free, server-first experiment with an explicit generated route
table and a small standard-library HTTP server.

### Architectural comparison

| Area | Ecosystem-first approach | antixt |
|---|---|---|
| Server | Async ecosystem built around Tokio, Hyper, HTTP, Tower, and Axum | Standard-library, thread-per-connection server |
| Routing | Rust module attributes, inventory discovery, layouts, groups, layers, and manual registration | Next-style `app/**/page.rs` files and an explicit generated route table |
| Views | HTML-shaped procedural macros, async components, and a macro-aware formatter | Ordinary Rust builders plus the `view!` declarative macro, formatted by rustfmt |
| Client model | Optional signals whose Rust expressions are also translated to JavaScript, plus procedures and shards | Zero JavaScript by default, with explicit fragments and JavaScript islands |
| Assets | Compile-time declarations discovered in the binary, then content-hashed into a manifest | Embedded framework client modules; no general asset pipeline yet |
| CSS and UI | Optional Tailwind integration and a source-vendored UI component registry | Rust-typed utility values with render-time dead-style elimination |
| Dependency policy | Feature-gated multi-crate workspace with a broad ecosystem surface | No third-party runtime or build dependencies |
| Default JavaScript | None for non-interactive pages; the reactive runtime is opt-in | None for ordinary pages; the 2.4 KB enhancement runtime is opt-in |

One novel pattern is a dual-target reactive expression. An expression is
type-checked and evaluated as Rust for the first server render, then translated
by a procedural macro into JavaScript for browser updates. This avoids a WASM
bundle and a separate client build, but it cannot support arbitrary Rust. The
shared vocabulary must restrict values, operators, methods, and control flow,
while an escape hatch to handwritten JavaScript leaves the developer responsible
for keeping server and browser implementations semantically aligned.

That model can be extended with typed server functions and server-rendered
reactive regions. Server functions generate POST endpoints callable from the
browser. Reactive regions re-render when signal arguments change, coalesce
same-tick updates, and abort stale requests. Replacing a region resets state
inside it, and its input remains untrusted network input despite its generated
Rust type.

### What antixt should borrow

1. **Typed application state and request-scoped memoization.** Shared services
   can be registered and retrieved by Rust type. A request-local cache can also
   deduplicate concurrent async work. antixt should add both concepts, but report
   duplicate or missing registrations as startup/request errors where possible
   rather than panicking.

2. **A production server boundary.** An ecosystem-based server inherits mature
   protocol handling, middleware composition, cancellation, sessions,
   compression, cookies, and multipart support. antixt should keep its small
   standard-library engine for learning and measurement, while defining a stable
   interface that can support an optional production backend. A dependency-free
   core should not require reimplementing every hardened HTTP concern forever.

3. **Content-addressed assets.** A native asset pipeline can create
   content-hashed names, a manifest, immutable caching, stale-file cleanup,
   parallel processing, and checksum-pinned remote assets. antixt should adopt
   those outcomes. Initially it should prefer an explicit generated registry or
   an `assets/` scan over the more opaque technique of scanning compiled binaries
   for static declarations.

4. **Typed server actions and latest-request-wins fragments.** antixt's fragment
   protocol can evolve into generated, typed server procedures. Its client
   runtime should coalesce updates and cancel or ignore stale responses. This
   captures the best part of this approach without first requiring a
   Rust-to-JavaScript expression compiler.

5. **Route groups, route-local layers, and an escape hatch.** Module-derived
   routers demonstrate groups that organize code without changing the URL,
   layout nesting, route-local request layers, API handlers, and manual
   registration. antixt should add equivalent file conventions while retaining
   its transparent filesystem routing and generated route table.

6. **Source-owned typed UI components.** A vendored component CLI can copy
   components into the application, track them in a manifest, and leave them as
   editable Rust source. An `antixt ui add button` workflow should combine that
   ownership model with antixt's typed utilities instead of copying Tailwind
   class strings. Update provenance and a safe diff/merge story will be important
   once users edit the generated files.

7. **Parity-first framework benchmarks.** A strong cross-framework suite should
   implement the same storefront in every framework, verify visible-output
   parity before measuring, test both saturation and a fixed request rate,
   record response size and metadata, and include one-core as well as all-core
   Rust runs. antixt should use this methodology rather than relying only on tiny
   synthetic landing pages.

8. **Context-aware output safety.** Generated JavaScript and serialized signal
   values need distinct output contexts with tests for comment and quote
   breakout. antixt already escapes text and attribute values, but any future
   script, style, URL, raw-HTML, or serialized-data APIs should use separate Rust
   types and context-specific encoders rather than one generic escape function.

9. **A formatter only when the syntax earns it.** A narrow formatter can target
   known view macros and integrate with editors. It is much smaller than
   maintaining a custom LSP, but it is still another parser and tool to support.
   antixt should continue designing `view!` for rustfmt and rust-analyzer, and
   revisit a macro formatter only if HTML-like ergonomics clearly justify the
   maintenance cost.

### Pros and costs of the ecosystem-first approach

| Advantages over antixt today | Costs relative to antixt |
|---|---|
| Production-oriented async networking and middleware ecosystem | More dependencies, compile work, supply-chain surface, and architectural complexity |
| Async components can query application services directly | Broad procedural-macro and inventory machinery makes generated behavior less explicit |
| Local reactive updates without WASM or a separate client bundler | The restricted Rust-to-JavaScript grammar is a second language surface with semantic limits |
| Typed procedures and server-rendered reactive shards | Opt-in browser APIs introduce serialization, authorization, cancellation, and state-reset concerns |
| Content-hashed assets, fonts, icons, Tailwind, sessions, and vendored UI | Binary-scanned assets and linker inventory are harder to inspect than generated source tables |
| Realistic cross-framework benchmark design | Several prominent capabilities remain experimental or are still roadmap items |

### Pros and costs of antixt's approach

| Advantages over the ecosystem-first approach | Current costs |
|---|---|
| Small, attributable, dependency-free core | The custom HTTP server is explicitly not production hardened |
| Ordinary Rust works with rust-analyzer and rustfmt without a new grammar | No typed app state, request cache, mature middleware, sessions, or multipart handling yet |
| Next-style file routing produces an explicit route table | Fewer route composition features and no manual adapter boundary yet |
| Zero JavaScript by default with a tiny explicit enhancement runtime | Rich interaction currently needs handwritten islands or server round trips |
| Fast no-change builds, small server binary, and easy benchmark attribution | Current benchmarks are more synthetic and less production representative |
| Typed utilities generate only used CSS | No general asset pipeline or source-vendored component registry yet |

The recommended direction is inspiration rather than convergence. antixt should
preserve ordinary Rust, file-based routing, zero-JavaScript pages, a small
auditable core, and measured feature costs. The most valuable external ideas are
a production backend boundary, typed app/request context, request-local
deduplication, content-addressed assets, typed server actions, route groups and
layers, source-owned typed components, and parity-checked benchmarks. The dual
Rust/JavaScript compiler is impressive, but it conflicts with antixt's decision
not to create a second language or custom editor stack and should remain a
deferred experiment rather than the framework's foundation.

## Benchmark limitations

These results are local synthetic measurements, not universal language rankings.
They are sensitive to machine state, process startup, compiler version, cache
warmth, fixture design, and binary settings. The canonical static-render test
does not measure TLS, database access, or sustained memory pressure. The scale
test adds request parsing, concurrency, latency, and memory, but it is a short
localhost run against tiny responses and a deliberately minimal HTTP stack.

## Recommended next research

1. Define a server-backend interface and benchmark an optional bounded async
   production engine against the standard-library reference engine.
2. Add typed application context and request-scoped memoization with concurrent
   work deduplication.
3. Build a content-hashed asset manifest with immutable caching, stale cleanup,
   integrity metadata, and CSP-compatible stylesheet extraction.
4. Evolve fragments into typed server actions with authorization hooks,
   coalescing, cancellation, and stale-response suppression.
5. Add pathless route groups, route-local middleware/layers, typed route errors,
   multipart forms, and an explicit manual-router escape hatch.
6. Prototype a source-owned `antixt ui add` registry whose components compose
   typed utilities and retain update provenance.
7. Build a parity-checked storefront benchmark against Next.js, another
   full-stack Rust framework, and a minimal Rust baseline; measure one-core and
   all-core saturation, fixed-rate latency, response bytes, memory, and raw
   result metadata.
8. Build dependency-directed debug invalidation and preserve island state across
   development reloads.
9. Audit context-specific escaping, HTTP correctness, slow clients, proxies,
   security limits, and sustained load before considering production use.
10. Compare the Rust HTML macro against a larger application for ergonomics,
    accessibility, compile-error quality, and whether rustfmt remains sufficient.
