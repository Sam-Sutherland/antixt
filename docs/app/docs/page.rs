use crate::components::docs::{callout, docs_page, section};
use antixt::css::{self, Breakpoint, u};
use antixt::{Context, Html, html, view};

pub fn page(_context: Context<'_>) -> Html {
    let cards = html::div()
        .styles([
            u::GRID,
            u::GRID_COLS_1,
            u::GAP_4,
            css::at(Breakpoint::Medium, u::GRID_COLS_2),
        ])
        .child(card(
            "/docs/quick-start",
            "Quick start",
            "Create, check, develop, and build your first app.",
        ))
        .child(card(
            "/docs/routing",
            "File routing",
            "Pages, layouts, actions, dynamic segments, and catch-alls.",
        ))
        .child(card(
            "/docs/html-components",
            "HTML & components",
            "Escaped builders, view!, props, and normal Rust modules.",
        ))
        .child(card(
            "/docs/fragments-islands",
            "Fragments & islands",
            "Add focused browser behavior without hydrating the app.",
        ));
    let content = html::fragment()
        .child(section(
            "what-is-antixt",
            "What is antixt?",
            html::fragment()
                .child(html::p().text(
                    "antixt is an experimental server-first framework in which both the framework and application are written in Rust. It borrows the productive filesystem conventions of Next.js without making Node or JavaScript the application runtime.",
                ))
                .child(callout(
                    "info",
                    "No custom language",
                    "antixt scans filenames but never parses your Rust source. rustc validates generated route wrappers, and rust-analyzer provides the editor experience.",
                )),
        ))
        .child(section("choose-a-path", "Choose a path", cards))
        .child(section(
            "status",
            "Project status",
            html::fragment()
                .child(html::p().text(
                    "v0.3 proves the major framework boundaries: routing, requests, responses, async work, streaming, HTML composition, development reload, fragments, islands, and native builds.",
                ))
                .child(callout(
                    "warning",
                    "Research runtime",
                    "The current HTTP implementation is not production hardened. Treat antixt as a serious prototype while bounded concurrency, TLS/proxy testing, middleware, caching, and observability are developed.",
                )),
        ));
    docs_page(
        "overview",
        "Documentation",
        "antixt fundamentals",
        "Learn the model once: ordinary Rust in, native server out, JavaScript only where you explicitly ask for it.",
        content,
    )
}

fn card(href: &str, title: &str, copy: &str) -> Html {
    view! {
        a [href = href, styles = [
            u::P_4,
            u::BORDER,
            u::ROUNDED,
            u::BG_PANEL,
            u::NO_UNDERLINE,
            css::hover(u::BORDER_ACCENT),
        ]] {
            strong [styles = [u::BLOCK]] { (title) }
            span [styles = [u::TEXT_MUTED, u::TEXT_SM]] { (copy) }
        }
    }
}
