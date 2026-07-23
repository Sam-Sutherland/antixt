use crate::components::docs::{callout, code_block, docs_page, section};
use crate::components::theme;
use antixt::css::{self, u};
use antixt::{Context, Html, Value, html};

pub struct Params<'a> {
    pub slug: Value<'a>,
}

pub fn page(_context: Context<'_>, params: Params<'_>) -> Html {
    let slug = params.slug.decode().unwrap_or_default();
    match slug.as_ref() {
        "quick-start" => quick_start(),
        "routing" => routing(),
        "requests" => requests(),
        "state-caching" => state_caching(),
        "async-streaming" => async_streaming(),
        "html-components" => html_components(),
        "typed-css" => typed_css(),
        "fragments-islands" => fragments_islands(),
        "architecture" => architecture(),
        _ => not_found(slug.as_ref()),
    }
}

fn quick_start() -> Html {
    let content = html::fragment()
        .child(section(
            "requirements",
            "Requirements",
            html::fragment()
                .child(html::p().text(
                    "antixt requires a recent stable Rust toolchain and Cargo. Install the native CLI directly with Cargo; Node and pnpm are not part of the framework toolchain.",
                ))
                .child(code_block(
                    "Verify toolchain",
                    "rustc --version\ncargo --version",
                )),
        ))
        .child(section(
            "create",
            "Create an application",
            html::fragment()
                .child(html::p().text(
                    "The create command writes a managed application into the gitignored .apps directory, making experiments safe to destroy and recreate.",
                ))
                .child(code_block(
                    "Terminal",
                    "cargo install --git https://github.com/Sam-Sutherland/antixt antixt\nantixt create hello-antixt\nantixt routes .apps/hello-antixt\nantixt check .apps/hello-antixt",
                )),
        ))
        .child(section(
            "develop",
            "Run the development server",
            html::fragment()
                .child(code_block(
                    "Terminal",
                    "antixt dev .apps/hello-antixt --port 3000",
                ))
                .child(html::p().text(
                    "antixt fingerprints Rust and client JavaScript sources, performs an incremental Cargo build, keeps the previous valid server alive while compiling, then restarts and reloads after success.",
                )),
        ))
        .child(section(
            "production",
            "Build for production",
            html::fragment()
                .child(code_block(
                    "Terminal",
                    "antixt build .apps/hello-antixt\n# output: .apps/hello-antixt/.antixt/target/release/antixt-app",
                ))
                .child(callout(
                    "info",
                    "Native output",
                    "The result is a native executable. Development reload JavaScript is excluded, while opted-in fragment and island support remains available.",
                )),
        ));
    docs_page(
        "quick-start",
        "Get productive",
        "Quick start",
        "Create a pure-Rust web application, inspect its routes, and produce a native release server.",
        content,
    )
}

fn routing() -> Html {
    let content = html::fragment()
        .child(section(
            "conventions",
            "Route conventions",
            html::fragment()
                .child(code_block(
                    "Application tree",
                    "app/\n  layout.rs                 shared layout\n  page.rs                   GET /\n  about/page.rs             GET /about\n  blog/[slug]/page.rs       GET /blog/:slug\n  docs/[...path]/page.rs    GET /docs/*path\n  api/status/get.rs         layout-free GET\n  newsletter/post.rs        POST /newsletter",
                ))
                .child(html::p().text(
                    "Pages receive ancestor layouts from the leaf directory back to app/layout.rs. Method handlers and get.rs return responses directly without page layouts.",
                )),
        ))
        .child(section(
            "dynamic",
            "Typed dynamic parameters",
            html::fragment()
                .child(html::p().text(
                    "A dynamic route exports a Params struct. Generated wiring populates its named fields, so a mismatch fails during normal Cargo checking.",
                ))
                .child(code_block(
                    "app/blog/[slug]/page.rs",
                    "use antixt::{Context, Html, Value, view};\n\npub struct Params<'a> {\n    pub slug: Value<'a>,\n}\n\npub fn page(_ctx: Context<'_>, params: Params<'_>) -> Html {\n    let slug = params.slug.decode().unwrap_or_default();\n    view! { h1 { text(slug) } }\n}",
                )),
        ))
        .child(section(
            "precedence",
            "Precedence and validation",
            html::fragment()
                .child(html::p().text(
                    "Static routes are ordered before dynamic routes, and dynamic routes before catch-alls. Catch-all segments must be final. Parameter names must be valid non-keyword Rust identifiers.",
                ))
                .child(callout(
                    "info",
                    "No source parser",
                    "antixt learns contracts from directories and filenames. Generated Rust wrappers—not a custom parser—ask rustc to validate the page, layout, and method signatures.",
                )),
        ));
    docs_page(
        "routing",
        "Filesystem as API",
        "File routing",
        "Use familiar page and layout conventions while keeping every route module ordinary Rust.",
        content,
    )
}

fn requests() -> Html {
    let content = html::fragment()
        .child(section(
            "context",
            "Request context",
            html::fragment()
                .child(html::p().text(
                    "Context borrows the current request and exposes its method, path, route parameters, query string, headers, cookies, form body, and raw bytes.",
                ))
                .child(code_block(
                    "Typed values",
                    "pub fn get(context: Context<'_>) -> Response {\n    let page = context\n        .query(\"page\")\n        .map(|value| value.parse::<u32>())\n        .transpose()?\n        .unwrap_or(1);\n\n    let theme = context.cookie(\"theme\");\n    Response::text(format!(\"page {page}\"))\n}",
                )),
        ))
        .child(section(
            "forms",
            "URL-encoded forms",
            html::fragment()
                .child(code_block(
                    "app/newsletter/post.rs",
                    "pub fn post(context: Context<'_>) -> Response {\n    let Some(email) = context.form(\"email\") else {\n        return Response::text(\"Missing email\")\n            .with_status(422);\n    };\n\n    Response::html(\n        html::p().text(format!(\n            \"Subscribed: {}\",\n            email.decode().unwrap_or_default(),\n        )),\n    )\n}",
                ))
                .child(callout(
                    "warning",
                    "Current boundary",
                    "v0.4 handles application/x-www-form-urlencoded bodies. Multipart uploads and configurable request size policies are next-stage work.",
                )),
        ))
        .child(section(
            "responses",
            "Responses and redirects",
            html::fragment()
                .child(html::p().text(
                    "Return Html for the common path or construct a Response to control status, content type, headers, redirects, and body strategy.",
                ))
                .child(code_block(
                    "Response builders",
                    "Response::text(\"created\")\n    .with_status(201)\n    .header(\"X-Request-Id\", \"abc\");\n\nResponse::redirect(\"/complete\");",
                )),
        ));
    docs_page(
        "requests",
        "Server primitives",
        "Requests & responses",
        "Decode untrusted input explicitly, parse it into Rust types, and construct predictable responses.",
        content,
    )
}

fn state_caching() -> Html {
    let content = html::fragment()
        .child(section(
            "configuration",
            "Typed application state",
            html::fragment()
                .child(html::p().text(
                    "An optional app/config.rs module registers long-lived services before the server accepts requests. Values are indexed by their Rust TypeId, so retrieval needs no string keys or global mutable singleton.",
                ))
                .child(code_block(
                    "app/config.rs",
                    "use antixt::{Application, StartupError};\n\npub fn configure(app: &mut Application)\n    -> Result<(), StartupError>\n{\n    app.state(Database::connect())?;\n    app.lifecycle(RequestMetrics::default());\n    Ok(())\n}",
                ))
                .child(callout(
                    "info",
                    "Configuration errors stay explicit",
                    "Registering the same Rust type twice returns StartupError. Context::state returns StateError when a service is absent, keeping framework configuration failures inspectable instead of hiding them behind a panic.",
                )),
        ))
        .child(section(
            "memoization",
            "One cache per request",
            html::fragment()
                .child(html::p().text(
                    "Pages and ancestor layouts receive clones of the same Context and therefore share state, cancellation, timing, and memoized values. The cache is discarded when that request finishes; it never leaks user-specific data into another request.",
                ))
                .child(code_block(
                    "Shared data load",
                    "let catalog = context\n    .state::<Catalog>()\n    .expect(\"Catalog is configured\");\n\nlet products = context\n    .memoize_sync(\"featured\", || catalog.featured())\n    .expect(\"request is active\");\n\n// Async callers use the same typed cache and concurrent work is deduplicated.\nlet user = context\n    .memoize((\"user\", id), || database.user(id))\n    .await\n    .expect(\"request is active\");",
                ))
                .child(html::p().text(
                    "Cache identity includes both the key type and value type. Results are returned as Arc<T>, allowing pages, layouts, and concurrent futures to reuse the same allocation.",
                )),
        ))
        .child(section(
            "lifecycle",
            "Lifecycle and cancellation",
            html::fragment()
                .child(html::p().text(
                    "Context exposes request_id, elapsed, cancellation, and is_cancelled. RequestLifecycle observers receive start and finish events with method, path, status, elapsed time, cancellation, and client-disconnect state.",
                ))
                .child(code_block(
                    "Cooperative cancellation",
                    "let cancellation = context.cancellation();\n\nif cancellation.is_cancelled() {\n    return Response::text(\"cancelled\")\n        .with_status(499);\n}\n\ncancellation.cancelled().await;",
                ))
                .child(callout(
                    "warning",
                    "Cooperative today",
                    "The dependency-free server reports write-side disconnects and wakes memo waiters, but it cannot pre-empt arbitrary blocking Rust work. Handlers and data clients must observe the token; a future production backend can connect it to transport-level cancellation.",
                )),
        ));
    docs_page(
        "state-caching",
        "Request-scoped data",
        "State & caching",
        "Register services once, deduplicate repeated work within a request, and observe request lifecycle without global state.",
        content,
    )
}

fn async_streaming() -> Html {
    let content = html::fragment()
        .child(section(
            "async",
            "Async handlers",
            html::fragment()
                .child(html::p().text(
                    "async_response boxes a normal Rust future and converts its eventual output through IntoResponse. antixt's small executor parks the request thread and honours future wakeups.",
                ))
                .child(code_block(
                    "app/api/status/get.rs",
                    "use antixt::{AsyncResponse, Context, Response, async_response, sleep};\nuse std::time::Duration;\n\npub fn get(_ctx: Context<'_>) -> AsyncResponse<'_> {\n    async_response(async {\n        sleep(Duration::from_millis(2)).await;\n        Response::text(\"ready\")\n    })\n}",
                )),
        ))
        .child(section(
            "streaming",
            "Chunked streaming",
            html::fragment()
                .child(html::p().text(
                    "Response::stream accepts a Send iterator and writes each yielded string as an HTTP/1.1 chunk, flushing between chunks.",
                ))
                .child(code_block(
                    "app/stream/get.rs",
                    "pub fn get(_ctx: Context<'_>) -> Response {\n    Response::stream(\n        \"text/html; charset=utf-8\",\n        [\"<p>first</p>\", \"<p>second</p>\"],\n    )\n}",
                )),
        ))
        .child(section(
            "runtime",
            "Runtime trade-offs",
            callout(
                "warning",
                "Deliberately small",
                "The executor proves async contracts without a dependency. The server currently creates one OS thread per connection; bounded scheduling, cancellation, backpressure, and sustained-load testing remain required.",
            ),
        ));
    docs_page(
        "async-streaming",
        "Native data flow",
        "Async & streaming",
        "Use standard futures for deferred work and stream response chunks without buffering the complete body.",
        content,
    )
}

fn html_components() -> Html {
    let content = html::fragment()
        .child(section(
            "builders",
            "Escaped HTML builders",
            html::fragment()
                .child(html::p().text(
                    "Html is a typed node tree. text() and attr() escape their values by default; a void element rejects children, and document() emits the doctype.",
                ))
                .child(code_block(
                    "Builder style",
                    "html::main()\n    .styles([u::GRID, u::P_4, u::GAP_2])\n    .child(html::h1().text(title))\n    .child(html::p().text(user_supplied_copy))",
                )),
        ))
        .child(section(
            "view-macro",
            "The view! macro",
            html::fragment()
                .child(html::p().text(
                    "view! is a declarative macro, not a template language. Attributes use a checked allow-list, styles accepts typed utilities, and child expressions convert through IntoHtml. Html, escaped strings, Option, arrays, Vec, and collected iterators compose without framework-specific syntax.",
                ))
                .child(code_block(
                    "Macro style",
                    "use antixt::css::u;\n\nlet links = routes.iter()\n    .map(nav_link)\n    .collect::<Html>();\n\nview! {\n    main [\n        aria_label = \"Documentation\",\n        styles = [u::GRID, u::P_4, u::GAP_2],\n    ] {\n        h1 { (title) }\n        (show_intro.then(|| intro(copy)))\n        (links)\n    }\n}",
                )),
        ))
        .child(section(
            "components",
            "Components are functions",
            html::fragment()
                .child(html::p().text(
                    "A component is a regular function accepting a regular Rust struct. Lifetimes, generics, enums, traits, tests, refactoring, and navigation all work without framework-specific tooling.",
                ))
                .child(code_block(
                    "components/feature.rs",
                    "pub struct FeatureProps<'a> {\n    pub title: &'a str,\n    pub copy: &'a str,\n}\n\npub fn feature(props: FeatureProps<'_>) -> Html {\n    view! {\n        article {\n            h2 { (props.title) }\n            p { (props.copy) }\n        }\n    }\n}",
                )),
        ));
    docs_page(
        "html-components",
        "Rust-shaped UI",
        "HTML & components",
        "Compose escaped server HTML with builders or a concise macro while retaining the complete Rust toolchain.",
        content,
    )
}

fn typed_css() -> Html {
    let content = html::fragment()
        .child(section(
            "utilities",
            "Utilities as Rust values",
            html::fragment()
                .child(html::p().text(
                    "antixt exposes utilities as Rust constants under css::u. Typing u:: triggers rust-analyzer completion; u::P_2 and u::M_4 render as p-2 and m-4. A misspelling is a compiler error rather than a silently missing style.",
                ))
                .child(code_block(
                    "Typed utility composition",
                    "use antixt::css::u;\nuse antixt::view;\n\nview! {\n    div [styles = [\n        u::FLEX,\n        u::ITEMS_CENTER,\n        u::GAP_3,\n        u::FLEX_WRAP,\n        u::PX_4,\n        u::PY_2,\n    ]] {\n        \"Typed and terse.\"\n    }\n}",
                ))
                .child(callout(
                    "info",
                    "Dogfooded here",
                    "The antixt docs application now contains no semantic class attributes. Its shell, navigation, cards, spacing, responsive layout, and states are composed from built-in and project-level Rust utilities.",
                )),
        ))
        .child(section(
            "variants",
            "State and responsive variants",
            html::fragment()
                .child(html::p().text(
                    "Variants wrap the same Utility value. They remain composable Rust while producing familiar Tailwind-style output such as hover:bg-raised and md:grid.",
                ))
                .child(code_block(
                    "Variants",
                    "use antixt::css::{self, Breakpoint, u};\n\nhtml::article().styles([\n    u::BLOCK,\n    u::P_4,\n    css::hover(u::BG_RAISED),\n    css::focus_visible(u::TEXT_ACCENT),\n    css::at(Breakpoint::Medium, u::GRID),\n])",
                )),
        ))
        .child(section(
            "output",
            "Only used CSS is emitted",
            html::fragment()
                .child(html::p().text(
                    "Built-ins emit stable readable classes; lower-level constructor utilities retain deterministic hashes. Html::render walks the completed tree, removes duplicate utilities, and injects only the rules the page uses. HTML fragments carry their own required rules.",
                ))
                .child(code_block(
                    "Generated output",
                    "<div class=\"flex p-4 hover:bg-raised md:grid\">…</div>\n\n<style data-antixt-css>\n.flex{display:flex}\n.p-4{padding:1rem}\n.hover\\:bg-raised:hover{background:var(--raised)}\n@media (min-width:48rem){.md\\:grid{display:grid}}\n</style>",
                ))
                .child(callout(
                    "warning",
                    "Current boundary",
                    "Built-ins cover the common layout vocabulary and Utility::named supports autocomplete-friendly project tokens. Conflict diagnostics, class merging, container queries, and production CSS extraction remain future work.",
                )),
        ));
    docs_page(
        "typed-css",
        "Compile-time styling",
        "Typed utility CSS",
        "Tailwind-like composition with rust-analyzer autocomplete, readable output classes, typed variants, and page-level dead-style elimination.",
        content,
    )
}

fn fragments_islands() -> Html {
    let content = html::fragment()
        .child(section(
            "default",
            "Zero JavaScript by default",
            html::fragment()
                .child(html::p().text(
                    "A normal production page contains no framework JavaScript. antixt injects its small browser runtime only when rendered HTML contains a fragment or island marker.",
                ))
                .child(callout(
                    "info",
                    "Progressive by construction",
                    "Links and forms remain normal HTML. Enhancement changes the response swap behavior; it does not replace server routing or rendering.",
                )),
        ))
        .child(section(
            "fragments",
            "HTML fragments",
            html::fragment()
                .child(code_block(
                    "Enhanced form",
                    "html::form()\n    .attr(\"method\", \"post\")\n    .attr(\"action\", \"/newsletter\")\n    .fragment_form()\n    .fragment_target(\"#result\")",
                ))
                .child(html::p().text(
                    "The browser sends antixt-fragment: true, receives ordinary server-rendered HTML, and swaps the target's contents. fragment_get, fragment_post, and fragment_swap cover link and button interactions.",
                )),
        ))
        .child(section(
            "islands",
            "Embedded client islands",
            html::fragment()
                .child(code_block(
                    "Rust markup",
                    "html::div()\n    .island(\"counter\")\n    .child(html::button().text(\"Count: 0\"))",
                ))
                .child(code_block(
                    "client/counter.js",
                    "export default function mount(root) {\n  const button = root.querySelector('button');\n  button.addEventListener('click', () => {\n    // focused browser behavior\n  });\n}",
                ))
                .child(html::p().text(
                    "Client modules are embedded with include_str! in the native binary and served from /__antixt/client. The runtime imports only the modules present on the page.",
                )),
        ));
    docs_page(
        "fragments-islands",
        "Progressive enhancement",
        "Fragments & islands",
        "Keep server HTML as the foundation, then add focused browser behavior without hydrating the application tree.",
        content,
    )
}

fn architecture() -> Html {
    let content = html::fragment()
        .child(section(
            "ownership",
            "Who owns what?",
            html::fragment()
                .child(html::p().text(
                    "rustc and rust-analyzer own language syntax, types, diagnostics, formatting, completion, navigation, and refactoring. antixt owns filesystem conventions, generated route wiring, HTML, the HTTP boundary, development supervision, and production builds.",
                ))
                .child(code_block(
                    "Pipeline",
                    "app/**/*.rs\n    ↓ filename scan\n.antixt/generated/main.rs\n    ↓ cargo + rustc\nnative antixt-app binary\n    ↓ HTTP\nHTML, fragments, optional islands",
                )),
        ))
        .child(section(
            "modules",
            "Framework modules",
            html::ul()
                .child(html::li().text("project.rs — route and client discovery"))
                .child(html::li().text("codegen.rs — typed wrappers and route tables"))
                .child(html::li().text("html.rs — node tree, escaping, and view!"))
                .child(html::li().text("css.rs — typed tokens and atomic utility rules"))
                .child(html::li().text(
                    "server.rs — typed state, request scopes, futures, streams, and HTTP",
                ))
                .child(html::li().text("dev.rs — source fingerprints and reload supervision"))
                .child(html::li().text("tooling.rs — generated source and Cargo orchestration")),
        ))
        .child(section(
            "dependencies",
            "Why dependency-free today?",
            html::fragment()
                .child(html::p().text(
                    "The current constraint makes costs visible and keeps the benchmark attributable. It is a research tool, not a permanent ban: production HTTP correctness, cryptography, and TLS should prefer proven implementations when antixt reaches those boundaries.",
                ))
                .child(callout(
                    "warning",
                    "Safety over ideology",
                    "No dependencies is useful while discovering the architecture. It should never justify reimplementing security-critical machinery poorly.",
                )),
        ));
    docs_page(
        "architecture",
        "Under the hood",
        "Architecture",
        "A thin framework layer around Rust's existing compiler, language server, build graph, and native output.",
        content,
    )
}

fn not_found(slug: &str) -> Html {
    let content = section(
        "missing",
        "Guide not found",
        html::fragment()
            .child(html::p().text(format!(
                "There is no documentation guide named `{slug}` in antixt v0.4.",
            )))
            .child(
                html::a()
                    .attr("href", "/docs")
                    .styles([
                        u::INLINE_FLEX,
                        u::ITEMS_CENTER,
                        u::JUSTIFY_CENTER,
                        theme::MIN_H_12,
                        u::PX_4,
                        u::PY_3,
                        u::BORDER,
                        u::BORDER_ACCENT,
                        u::ROUNDED,
                        u::BG_ACCENT,
                        theme::TEXT_INK,
                        u::NO_UNDERLINE,
                        u::FONT_BOLD,
                        css::hover(theme::TRANSLATE_Y_NEG_05),
                    ])
                    .text("Return to documentation"),
            ),
    );
    docs_page(
        "missing",
        "404-ish",
        "Unknown guide",
        "The dynamic route matched, but this documentation slug is not defined.",
        content,
    )
}
