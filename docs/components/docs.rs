use crate::components::theme;
use antixt::css::{self, Breakpoint, u};
use antixt::{Html, html, view};

const NAVIGATION: &[(&str, &str, &str)] = &[
    ("overview", "/docs", "Overview"),
    ("quick-start", "/docs/quick-start", "Quick start"),
    ("routing", "/docs/routing", "File routing"),
    ("requests", "/docs/requests", "Requests & responses"),
    (
        "async-streaming",
        "/docs/async-streaming",
        "Async & streaming",
    ),
    ("state-caching", "/docs/state-caching", "State & caching"),
    (
        "html-components",
        "/docs/html-components",
        "HTML & components",
    ),
    ("typed-css", "/docs/typed-css", "Typed utility CSS"),
    (
        "fragments-islands",
        "/docs/fragments-islands",
        "Fragments & islands",
    ),
    ("architecture", "/docs/architecture", "Architecture"),
];

pub fn docs_page(active: &str, eyebrow: &str, title: &str, lede: &str, content: Html) -> Html {
    view! {
        div [styles = [
            u::GRID,
            u::GRID_COLS_1,
            theme::GAP_DOCS,
            u::ITEMS_START,
            u::PY_16,
            css::at(Breakpoint::Large, theme::GRID_COLS_DOCS),
        ]] {
            (sidebar(active))
            main [id = "main-content", styles = [u::MIN_W_0]] {
                header [styles = [u::PB_12, u::BORDER_B]] {
                    (eyebrow_text(eyebrow))
                    h1 [styles = [
                        u::M_0,
                        theme::TEXT_DOC_TITLE,
                        theme::LEADING_TIGHT,
                        theme::TRACKING_DOC_TITLE,
                    ]] { (title) }
                    p [styles = [
                        theme::MAX_W_2XL,
                        u::MT_6,
                        u::MB_0,
                        u::TEXT_MUTED,
                        theme::TEXT_12,
                    ]] { (lede) }
                }
                (content)
            }
        }
    }
}

fn eyebrow_text(copy: &str) -> Html {
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

fn sidebar(active: &str) -> Html {
    let navigation = NAVIGATION
        .iter()
        .map(|(key, href, label)| {
            let mut styles = vec![
                u::BLOCK,
                u::PX_3,
                u::PY_2,
                theme::BORDER_L_2,
                theme::BORDER_TRANSPARENT,
                u::TEXT_MUTED,
                u::NO_UNDERLINE,
                theme::TEXT_09,
                css::hover(u::TEXT_DEFAULT),
                css::hover(u::BG_PANEL),
            ];
            if *key == active {
                styles.extend([u::BORDER_ACCENT, u::TEXT_DEFAULT, theme::BG_ACCENT_SOFT]);
            }
            let link = view! {
                a [href = href, data_doc_title = label, styles = styles] { (*label) }
            };
            if *key == active {
                link.attr("aria-current", "page")
            } else {
                link
            }
        })
        .collect::<Html>();

    let search = html::div().island("docs-search").child(view! {
        div {
            label [for_ = "docs-search", styles = [
                u::BLOCK,
                u::MB_2,
                u::TEXT_MUTED,
                theme::TEXT_076,
            ]] { "Search documentation" }
            input [
                id = "docs-search",
                type_ = "search",
                placeholder = "Search docs…",
                autocomplete = "off",
                styles = [
                    u::W_FULL,
                    u::BORDER,
                    u::ROUNDED,
                    u::PX_3,
                    u::PY_3,
                    u::BG_PANEL,
                    u::TEXT_DEFAULT,
                    theme::OUTLINE_NONE,
                    css::focus_visible(u::BORDER_ACCENT),
                    css::focus_visible(theme::SHADOW_FOCUS),
                ],
            ] {}
            p [aria_live = "polite", styles = [
                theme::MIN_H_12,
                u::MT_1,
                u::MB_4,
                u::TEXT_MUTED,
                theme::TEXT_072,
            ]] {}
        }
    });

    view! {
        aside [styles = [
            u::GRID,
            u::GRID_COLS_1,
            u::GAP_4,
            theme::STATIC,
            css::at(Breakpoint::Medium, u::GRID_COLS_2),
            css::at(Breakpoint::Large, u::BLOCK),
            css::at(Breakpoint::Large, u::STICKY),
            css::at(Breakpoint::Large, theme::TOP_25),
        ]] {
            (search)
            p [styles = [
                u::MT_4,
                u::MB_2,
                theme::TEXT_DIM,
                u::UPPERCASE,
                theme::TRACKING_WIDE,
                u::FONT_MONO,
                u::FONT_BOLD,
                theme::TEXT_066,
            ]] { "Learn antixt" }
            nav [aria_label = "Documentation", styles = [
                u::GRID,
                u::GRID_COLS_1,
                u::GAP_1,
                css::at(Breakpoint::Medium, u::GRID_COLS_2),
                css::at(Breakpoint::Large, u::GRID_COLS_1),
            ]] { (navigation) }
            div [styles = [
                u::HIDDEN,
                u::MT_8,
                u::P_4,
                u::BORDER,
                u::ROUNDED,
                u::BG_PANEL,
                css::at(Breakpoint::Large, u::BLOCK),
            ]] {
                strong [styles = [u::TEXT_ACCENT, theme::TEXT_08]] { "v0.4 experimental" }
                p [styles = [u::MT_1, u::MB_0, u::TEXT_MUTED, theme::TEXT_078]] {
                    "A real framework, still a research runtime."
                }
            }
        }
    }
}

pub fn section(id: &str, title: &str, body: Html) -> Html {
    let anchor = format!("#{id}");
    view! {
        section [id = id, styles = [u::PY_12, u::BORDER_B]] {
            (html::h2()
                .styles([u::MT_0, u::MB_4, u::TEXT_XL, theme::TRACKING_TIGHT])
                .child(html::a().attr("href", anchor).styles([u::NO_UNDERLINE]).text(title)))
            (html::div().attr("data-doc-body", "").child(body))
        }
    }
}

pub fn code_block(label: &str, source: &str) -> Html {
    view! {
        div [styles = [u::MY_6, u::BORDER, u::ROUNDED_LG, theme::BG_CODE, u::OVERFLOW_HIDDEN]] {
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
                span { (label) }
                span [styles = [theme::TEXT_DOTS, theme::TRACKING_DOTS]] { "● ● ●" }
            }
            pre [styles = [u::M_0, u::P_4, u::OVERFLOW_X_AUTO, theme::TEXT_CODE, u::FONT_MONO, theme::TEXT_084, theme::LEADING_RELAXED]] {
                code { (source) }
            }
        }
    }
}

pub fn callout(kind: &str, title: &str, copy: &str) -> Html {
    let mut styles = vec![u::MY_6, u::P_4, theme::BORDER_L_3];
    if kind == "warning" {
        styles.extend([theme::BORDER_DANGER, theme::BG_DANGER_SOFT]);
    } else {
        styles.extend([theme::BORDER_CYAN, theme::BG_CYAN_SOFT]);
    }
    view! {
        aside [styles = styles] {
            strong [styles = [theme::TEXT_09]] { (title) }
            p [styles = [u::MT_1, u::MB_0]] { (copy) }
        }
    }
}

pub fn feature_link(href: &str, index: &str, title: &str, copy: &str) -> Html {
    view! {
        a [href = href, styles = [
            theme::MIN_H_CARD,
            u::P_8,
            u::BG_PANEL,
            u::NO_UNDERLINE,
            u::BORDER_R,
            u::BORDER_B,
            css::hover(u::BG_RAISED),
        ]] {
            span [styles = [u::TEXT_ACCENT, u::FONT_MONO, theme::TEXT_072]] { (index) }
            h3 [styles = [u::MT_8, u::MB_2, u::TEXT_XL]] { (title) }
            p [styles = [u::M_0, u::TEXT_MUTED, theme::MAX_W_LG]] { (copy) }
            span [styles = [u::BLOCK, u::MT_6, u::TEXT_CYAN, theme::TEXT_09]] { "Read guide →" }
        }
    }
}
