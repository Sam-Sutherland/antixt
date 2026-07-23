# antixt

> The Rust web framework after Next.

[![CI](https://github.com/Sam-Sutherland/antixt/actions/workflows/ci.yml/badge.svg)](https://github.com/Sam-Sutherland/antixt/actions/workflows/ci.yml)

antixt v0.4 is a dependency-free, server-first Rust web framework. Applications
are ordinary Rust, routes follow Next.js-style filesystem conventions, and
production pages ship no JavaScript unless they opt into fragments or islands.

Rust Analyzer owns parsing, completion, navigation, formatting, refactoring,
and diagnostics. antixt owns route discovery, typed HTML, nested layouts,
requests and responses, async and streamed handlers, development reload, and
native production builds.

The documentation site under `docs/` is itself an antixt application and
dogfoods dynamic routes, nested layouts, typed HTML, and an optional search
island.

## Quick start

```sh
cargo install --git https://github.com/Sam-Sutherland/antixt antixt

antixt create hello-antixt
antixt check .apps/hello-antixt
antixt routes .apps/hello-antixt
antixt dev .apps/hello-antixt
```

The installed `antixt` binary embeds the matching dependency-free framework
library into each generated app, so scaffolds do not depend on this repository,
Node, pnpm, or a registry download.

## Rust file routing

```text
app/
  layout.rs                   shared layout
  page.rs                     GET /
  blog/[slug]/page.rs         typed dynamic parameter
  docs/[...path]/page.rs      typed catch-all parameter
  api/status/get.rs           layout-free GET handler
  newsletter/post.rs          POST handler
client/
  counter.js                  optional embedded island
components/
  mod.rs                      ordinary Rust modules
```

A dynamic route declares the struct generated wiring will construct. Invalid
field names or handler signatures are compile errors:

```rust
use antixt::{Context, Html, Value, view};

pub struct Params<'a> {
    pub slug: Value<'a>,
}

pub fn page(_context: Context<'_>, params: Params<'_>) -> Html {
    view! {
        main {
            h1 { "Article" }
            code { text(params.slug.decode().unwrap_or_default()) }
        }
    }
}
```

`view!` is a regular declarative Rust macro. Attribute names are checked,
typed utilities attach with `styles = [...]`, and child expressions accept
escaped strings, `Option`, arrays, vectors, iterator collections, or component
`Html` through `IntoHtml`. Components remain normal Rust functions, so
rust-analyzer owns navigation, completion, types, and refactoring.

## Typed utility CSS

antixt exposes autocomplete-friendly atomic utilities under `css::u`. Rust uses
underscores in identifiers while the rendered HTML uses familiar kebab-case:
`u::P_2` becomes `p-2` and `u::ITEMS_CENTER` becomes `items-center`.

```rust
use antixt::css::{self, Breakpoint, u};

html::div().styles([
    u::FLEX,
    u::P_4,
    u::GAP_2,
    css::hover(u::BG_RAISED),
    css::at(Breakpoint::Medium, u::GRID),
])
```

Typing `u::` gives normal rust-analyzer completion, unknown utilities fail to
compile, and output stays readable (`flex p-4 gap-2 hover:bg-raised md:grid`).
Responsive/state variants remain Rust values and only used rules are emitted.
antixt still does not scan or parse application source.

## Typed requests and responses

`Context` provides route parameters, query values, URL-encoded forms, headers,
cookies, and raw request bodies. `Value::decode` handles percent encoding and
`Value::parse::<T>` uses Rust's normal `FromStr` types.

```rust
pub fn post(context: Context<'_>) -> Response {
    let Some(email) = context.form("email") else {
        return Response::text("Missing email").with_status(422);
    };
    Response::redirect(format!("/thanks?email={}", email.encoded()))
}
```

Responses support status codes, headers, redirects, full bodies, and real HTTP
chunked streams. Async handlers use `async_response`; antixt's small standard-
library executor honours normal future wakeups without bringing in a runtime:

```rust
pub fn get(_context: Context<'_>) -> AsyncResponse<'_> {
    async_response(async {
        sleep(Duration::from_millis(2)).await;
        Response::text("ready")
    })
}
```

## Typed state and request caching

`app/config.rs` configures shared services once at startup. Duplicate
registrations return a `StartupError`; missing services return a `StateError`
from the request context instead of panicking inside the framework.

```rust
pub fn configure(app: &mut Application) -> Result<(), StartupError> {
    app.state(Database::connect())?;
    app.lifecycle(RequestMetrics::default());
    Ok(())
}
```

Pages, components, and layouts share one request scope. `memoize_sync` supports
normal server rendering, while `memoize` deduplicates concurrent futures with
the same typed key and value. Both return an `Arc<T>` and observe cancellation.

```rust
let catalog = context
    .state::<Catalog>()
    .expect("Catalog is configured");
let products = context
    .memoize("featured", || catalog.featured())
    .await
    .expect("request is active");
```

`Context` also exposes a request ID, elapsed time, and cancellation token.
Lifecycle observers receive typed start and finish events including status,
latency, cancellation, and client-disconnect state.

### Upgrading layouts from v0.3

Layouts now receive the shared request context before their children:

```rust
pub fn layout(context: Context<'_>, children: Html) -> Html {
    // Pages and layouts can now reuse the same state and memoized values.
    view! { body { (children) } }
}
```

## Optional browser enhancement

`.fragment_form()`, `.fragment_get()`, and `.fragment_post()` request HTML
fragments and swap a selected target. `.island("counter")` mounts an embedded
`client/counter.js` module. antixt injects its 2.9 KB inline runtime only when an
HTML document contains an enhancement marker; ordinary pages remain zero-JS.

The canonical browser test verified a fragment form update, a stateful counter
island, and a dynamic route. No external client library or bundler is involved.

## Measured performance

The local seven-route fixture measured a 1,155 ms cold release build, 40 ms
no-change build, 267 ms application edit build, and 1.67 ms render-process
startup. The disposable 1,000-route fixture measured:

| Measurement | Result |
|---|---:|
| Route scan | 273.73 ms |
| Cold check | 456 ms |
| Warm check | 60 ms |
| Cold release build | 1,628 ms |
| No-change build | 60 ms |
| Shared leaf edit | 973 ms |
| Throughput, concurrency 50 | 16,452 req/s |
| HTTP p50 / p95 / p99 | 2.14 / 6.76 / 13.02 ms |
| Resident memory after load | 2.80 MB |

These are local synthetic measurements of a deliberately small HTTP stack, not
production capacity claims. See [RESEARCH.md](RESEARCH.md) for methodology and
limitations.

## Current boundary

antixt now proves all major framework seams, but its HTTP implementation remains
experimental. Before production use it needs bounded concurrency,
transport-level cancellation, TLS/proxy hardening, multipart forms, middleware,
cross-request caching, production observability adapters, dependency-directed
HMR, and broader protocol/security testing. The current cancellation token is
cooperative; the standard-library transport only detects a disconnect when a
response write fails.

The typed CSS spike additionally needs app-defined themes, conflict diagnostics,
container queries, stylesheet caching, and production extraction.

```sh
cargo test --manifest-path antixt/Cargo.toml
cargo clippy --manifest-path antixt/Cargo.toml --all-targets -- -D warnings
node benchmark/antixt-rust.mjs
node benchmark/antixt-scale.mjs
antixt dev docs --port 4174
```

## License

antixt is available under the [MIT License](LICENSE).
