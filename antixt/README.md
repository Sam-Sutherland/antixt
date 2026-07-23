# antixt framework crate

`antixt` v0.4 is a dependency-free Rust library and CLI for pure-Rust
web applications.

## Installation

From GitHub:

```sh
cargo install --git https://github.com/Sam-Sutherland/antixt antixt
antixt version
```

Framework contributors can instead install the current checkout with
`cargo install --path antixt` from the repository root.

The resulting `antixt` executable is self-contained. Generated applications
vendor its matching framework library under `.antixt/framework`, so they remain
buildable outside the antixt repository.

## Route contracts

- `page.rs`: GET page wrapped by ancestor `layout.rs` files.
- `get.rs`: layout-free GET response, including async or streamed responses.
- `post.rs`, `put.rs`, `patch.rs`, `delete.rs`: method handlers.
- `[slug]`: generated route-specific `Params { slug: Value<'a> }`.
- `[...path]`: final catch-all parameter.
- `client/**/*.js`: compile-time embedded optional island modules.
- `app/config.rs`: optional typed state and lifecycle configuration.

antixt scans names, never Rust syntax. It generates deterministic module imports,
typed wrappers, route patterns, and embedded client assets under
`<project>/.antixt/generated/main.rs`; rustc validates every application
contract. Only `.antixt/target/` is ignored.

## Runtime modules

- `html.rs`: escaped HTML tree, enhancement markers, and `view!` macro.
- `css.rs`: typed atomic utilities, semantic tokens, states, and breakpoints.
- `server.rs`: route matching, typed state, request memoization, cancellation,
  lifecycle observers, responses, async execution, streams, and HTTP serving.
- `project.rs`: route/client discovery, parameter validation, and specificity.
- `codegen.rs`: generated Rust module wiring and typed handler wrappers.
- `tooling.rs`: change-aware generation plus Cargo check/build orchestration.
- `dev.rs`: Rust/JavaScript fingerprinting, incremental build, and child reload.
- `main.rs`: lifecycle CLI and v0.4 scaffold.

## Commands

```text
antixt create <name> [--force]
antixt destroy <name> --force
antixt check [project]
antixt routes [project]
antixt build [project]
antixt dev [project] [--port N]
antixt run [project] [--port N]
```

```sh
cargo test --manifest-path antixt/Cargo.toml
cargo clippy --manifest-path antixt/Cargo.toml --all-targets -- -D warnings
```
