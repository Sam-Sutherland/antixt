use crate::components::theme;
use antixt::css::{self, Breakpoint, u};
use antixt::{Html, view};

const STYLES: &str = r###"
:root {
  color-scheme: dark;
  --bg: #080b0f;
  --panel: #0f141b;
  --raised: #151c25;
  --line: #28323d;
  --text: #f2f5f2;
  --muted: #98a59e;
  --accent: #b7f36b;
  --cyan: #72d7ff;
  --danger: #ffb38a;
  --font-sans: Space Grotesk, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, Segoe UI, sans-serif;
  --font-mono: IBM Plex Mono, ui-monospace, SFMono-Regular, Consolas, monospace;
  font-family: var(--font-sans);
  background: var(--bg);
  color: var(--text);
}
* { box-sizing: border-box; }
html { scroll-behavior: smooth; }
body { margin: 0; min-width: 320px; background: var(--bg); color: var(--text); line-height: 1.65; }
a { color: inherit; }
button, input { font: inherit; }
code, pre, kbd { font-family: var(--font-mono); }
::selection { background: var(--accent); color: #11170b; }
[hidden] { display: none !important; }
[data-doc-body] h3 { margin-top: 2rem; }
[data-doc-body] p, [data-doc-body] li { color: #bdc7c1; }
[data-doc-body] ul, [data-doc-body] ol { padding-left: 1.25rem; }
[data-doc-body] code:not(pre code) { padding: .15rem .35rem; border: 1px solid var(--line); border-radius: .3rem; background: var(--panel); color: var(--cyan); font-size: .86em; }
:focus-visible { outline: 2px solid var(--accent); outline-offset: 3px; }
"###;

pub fn layout(children: Html) -> Html {
    view! {
        document [lang = "en"] {
            head {
                meta [charset = "utf-8"] {}
                meta [name = "viewport", content = "width=device-width, initial-scale=1"] {}
                meta [
                    name = "description",
                    content = "antixt is a dependency-free, server-first Rust web framework.",
                ] {}
                link [rel = "preconnect", href = "https://fonts.googleapis.com"] {}
                link [
                    rel = "preconnect",
                    href = "https://fonts.gstatic.com",
                    crossorigin = "",
                ] {}
                link [
                    rel = "stylesheet",
                    href = "https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500;600&family=Space+Grotesk:wght@400;500;600;700&display=swap",
                ] {}
                title { "antixt — Rust-native web framework" }
                style { (STYLES) }
            }
            body {
                a [href = "#main-content", styles = [
                    theme::FIXED,
                    theme::LEFT_4,
                    theme::NEG_TOP_20,
                    theme::Z_50,
                    u::PX_4,
                    u::PY_3,
                    u::BG_ACCENT,
                    theme::TEXT_INK,
                    css::focus_visible(theme::TOP_4),
                ]] { "Skip to content" }
                header [styles = [
                    u::STICKY,
                    theme::TOP_0,
                    theme::Z_20,
                    theme::BORDER_B_SUBTLE,
                    theme::BG_TOPBAR,
                    theme::BACKDROP_BLUR,
                ]] {
                    div [styles = [
                        theme::W_PAGE,
                        theme::MIN_H_17,
                        u::MX_AUTO,
                        u::FLEX,
                        u::ITEMS_CENTER,
                        u::JUSTIFY_BETWEEN,
                        u::GAP_8,
                    ]] {
                        a [href = "/", aria_label = "antixt home", styles = [
                            u::INLINE_FLEX,
                            u::ITEMS_CENTER,
                            u::GAP_3,
                            u::NO_UNDERLINE,
                            theme::FONT_790,
                            theme::TRACKING_BRAND,
                        ]] {
                            span [styles = [
                                theme::W_8,
                                theme::H_8,
                                u::GRID,
                                theme::PLACE_ITEMS_CENTER,
                                u::BORDER,
                                u::BORDER_ACCENT,
                                u::BG_ACCENT,
                                theme::TEXT_INK,
                                u::FONT_MONO,
                                u::FONT_BLACK,
                                theme::TEXT_08,
                                theme::ROTATE_NEG_3,
                                theme::SHADOW_MARK,
                            ]] { "A" }
                            span { "antixt" }
                        }
                        nav [aria_label = "Primary", styles = [u::FLEX, u::ITEMS_CENTER, u::GAP_1]] {
                            (nav_link("/docs", "Docs", false))
                            (nav_link("/benchmarks", "Benchmarks", false))
                            (nav_link("/docs/architecture", "v0.3", true))
                        }
                    }
                }
                div [styles = [theme::W_PAGE, u::MX_AUTO]] { (children) }
                footer [styles = [u::BORDER_T]] {
                    div [styles = [
                        theme::W_PAGE,
                        u::MX_AUTO,
                        u::PY_8,
                        u::FLEX,
                        u::FLEX_COL,
                        u::JUSTIFY_BETWEEN,
                        u::GAP_8,
                        u::TEXT_MUTED,
                        theme::TEXT_08,
                        css::at(Breakpoint::Small, u::FLEX_ROW),
                    ]] {
                        span { "antixt v0.3 — ordinary Rust, native output." }
                        span { "Experimental by design." }
                    }
                }
            }
        }
    }
}

fn nav_link(href: &str, label: &str, version: bool) -> Html {
    let mut styles = vec![
        u::PX_3,
        u::PY_2,
        u::ROUNDED,
        u::TEXT_MUTED,
        u::NO_UNDERLINE,
        theme::TEXT_09,
        css::hover(u::TEXT_DEFAULT),
        css::hover(u::BG_RAISED),
        css::focus_visible(u::TEXT_DEFAULT),
        css::focus_visible(u::BG_RAISED),
    ];
    if version {
        styles.extend([u::BORDER, u::TEXT_ACCENT, u::FONT_MONO, u::TEXT_XS]);
    } else {
        styles.extend([u::HIDDEN, css::at(Breakpoint::Small, u::BLOCK)]);
    }
    view! { a [href = href, styles = styles] { (label) } }
}
