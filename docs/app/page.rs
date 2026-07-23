use crate::components::docs::{code_block, feature_link};
use crate::components::theme;
use antixt::css::{self, Breakpoint, u};
use antixt::{Context, Html, view};

pub fn page(_context: Context<'_>) -> Html {
    let features = [
        feature_link(
            "/docs/routing",
            "01",
            "File-based routing",
            "Static, dynamic, catch-all, page, and method routes derived from filenames.",
        ),
        feature_link(
            "/docs/requests",
            "02",
            "Typed server APIs",
            "Queries, forms, headers, cookies, redirects, and escaped responses.",
        ),
        feature_link(
            "/docs/async-streaming",
            "03",
            "Async and streaming",
            "Standard Rust futures and real HTTP chunked bodies without an external runtime.",
        ),
        feature_link(
            "/docs/fragments-islands",
            "04",
            "JavaScript on your terms",
            "HTML fragments and embedded islands only where the interface earns them.",
        ),
    ];

    view! {
        main [id = "main-content"] {
            section [styles = [
                u::GRID,
                u::GRID_COLS_1,
                u::ITEMS_CENTER,
                theme::GAP_FLUID,
                u::PY_24,
                css::at(Breakpoint::Large, theme::MIN_H_HERO),
                css::at(Breakpoint::Large, theme::GRID_COLS_HERO),
            ]] {
                div {
                    (eyebrow("The Rust web framework after Next."))
                    h1 [styles = [
                        theme::MAX_W_3XL,
                        u::M_0,
                        theme::TEXT_DISPLAY,
                        theme::LEADING_DISPLAY,
                        theme::TRACKING_DISPLAY,
                        theme::FONT_790,
                    ]] {
                        "Build for"
                        span [styles = [u::BLOCK, u::TEXT_ACCENT]] { "the metal." }
                    }
                    p [styles = [
                        theme::MAX_W_2XL,
                        u::MT_8,
                        u::MB_0,
                        u::TEXT_MUTED,
                        theme::TEXT_13,
                    ]] {
                        "antixt brings file-based routing, typed HTML, async streaming, and optional client islands to ordinary Rust—without a custom language or a Node runtime."
                    }
                    div [styles = [u::FLEX, u::ITEMS_CENTER, u::GAP_3, u::FLEX_WRAP, u::MT_8]] {
                        (button("/docs/quick-start", "Start building →", true))
                        (button("/benchmarks", "See benchmarks", false))
                    }
                }
                div [styles = [
                    u::RELATIVE,
                    u::BORDER,
                    u::ROUNDED_LG,
                    u::BG_PANEL,
                    theme::SHADOW_TERMINAL,
                    u::OVERFLOW_HIDDEN,
                    theme::ROTATE_1,
                ]] {
                    div [styles = [
                        u::FLEX,
                        u::ITEMS_CENTER,
                        u::JUSTIFY_BETWEEN,
                        u::PX_4,
                        u::PY_3,
                        u::BORDER_B,
                        theme::BG_CODE_BAR,
                        u::TEXT_MUTED,
                        u::FONT_MONO,
                        theme::TEXT_074,
                    ]] {
                        span { "app/blog/[slug]/page.rs" }
                        span [styles = [theme::TEXT_DOTS, theme::TRACKING_DOTS]] { "● ● ●" }
                    }
                    pre [styles = [u::M_0, u::P_6, u::OVERFLOW_X_AUTO, theme::TEXT_CODE_MUTED, u::FONT_MONO, theme::TEXT_084, theme::LEADING_LOOSE]] {
                        code {
                            "use antixt::{Context, Html, Value, view};\n\npub struct Params<'a> {\n    pub slug: Value<'a>,\n}\n\npub fn page(\n    _ctx: Context<'_>,\n    params: Params<'_>,\n) -> Html {\n    view! {\n        main {\n            h1 { \"Hello from Rust\" }\n            code { (params.slug.decode()?) }\n        }\n    }\n}"
                        }
                    }
                }
            }
            section [aria_label = "antixt performance highlights", styles = [
                u::GRID,
                u::GRID_COLS_1,
                u::BORDER_T,
                u::BORDER_B,
                css::at(Breakpoint::Medium, u::GRID_COLS_2),
                css::at(Breakpoint::Large, u::GRID_COLS_4),
            ]] {
                (proof("37 ms", "No-change release build"))
                (proof("1.76 ms", "Render process startup"))
                (proof("0 B", "JavaScript by default"))
                (proof("21", "Passing framework tests"))
            }
            section [styles = [u::PY_24]] {
                (section_heading(
                    "One language. Full stack.",
                    "The useful parts of a modern framework.",
                    "antixt owns the web-framework seams while rustc and rust-analyzer own the language. Every application file remains normal, navigable, refactorable Rust.",
                ))
                div [styles = [
                    u::GRID,
                    u::GRID_COLS_1,
                    u::BORDER,
                    u::ROUNDED_LG,
                    u::OVERFLOW_HIDDEN,
                    css::at(Breakpoint::Medium, u::GRID_COLS_2),
                ]] { (features) }
            }
            section [styles = [u::PY_24]] {
                (section_heading(
                    "Start small",
                    "From empty folder to native server.",
                    "The framework, CLI, runtime, HTML renderer, async executor, and client enhancement layer currently use no third-party Rust crates.",
                ))
                (code_block(
                    "Terminal",
                    "cargo install --git https://github.com/Sam-Sutherland/antixt antixt\nantixt create hello-antixt\nantixt dev .apps/hello-antixt\n\n# production\nantixt build .apps/hello-antixt",
                ))
            }
        }
    }
}

fn eyebrow(copy: &str) -> Html {
    view! {
        p [styles = [
            u::MT_0,
            u::MB_4,
            u::TEXT_ACCENT,
            u::UPPERCASE,
            theme::TRACKING_WIDER,
            u::FONT_MONO,
            u::FONT_BOLD,
            theme::TEXT_076,
        ]] { (copy) }
    }
}

fn button(href: &str, label: &str, primary: bool) -> Html {
    let mut styles = vec![
        u::INLINE_FLEX,
        u::ITEMS_CENTER,
        u::JUSTIFY_CENTER,
        theme::MIN_H_12,
        u::PX_4,
        u::PY_3,
        u::BORDER,
        u::ROUNDED,
        u::NO_UNDERLINE,
        u::FONT_BOLD,
        css::hover(theme::TRANSLATE_Y_NEG_05),
    ];
    if primary {
        styles.extend([u::BORDER_ACCENT, u::BG_ACCENT, theme::TEXT_INK]);
    } else {
        styles.push(u::BG_PANEL);
    }
    view! { a [href = href, styles = styles] { (label) } }
}

fn section_heading(eyebrow_copy: &str, title: &str, copy: &str) -> Html {
    view! {
        div [styles = [theme::MAX_W_3XL, u::MB_12]] {
            (eyebrow(eyebrow_copy))
            h2 [styles = [u::M_0, theme::TEXT_HEADING, theme::LEADING_NONE, theme::TRACKING_TIGHTER]] { (title) }
            p [styles = [theme::MAX_W_XL, u::TEXT_MUTED, u::TEXT_LG]] { (copy) }
        }
    }
}

fn proof(value: &str, label: &str) -> Html {
    view! {
        div [styles = [u::P_8, u::BORDER_R, u::BORDER_B]] {
            strong [styles = [u::BLOCK, theme::TEXT_2XL, theme::TRACKING_STAT]] { (value) }
            span [styles = [u::TEXT_MUTED, theme::TEXT_086]] { (label) }
        }
    }
}
