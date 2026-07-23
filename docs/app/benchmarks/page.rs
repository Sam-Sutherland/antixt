use crate::components::docs::{callout, docs_page, section};
use crate::components::theme;
use antixt::css::u;
use antixt::{Context, Html, html, view};

pub fn page(_context: Context<'_>) -> Html {
    let small = table(&[
        ("Cold release build", "1,155 ms"),
        ("No-change release build", "40 ms"),
        ("One-file application edit", "267 ms"),
        ("Render process", "1.67 ms"),
        ("Native server", "605,600 B"),
        ("Ordinary route JavaScript", "0 B"),
    ]);
    let storefront = table(&[
        ("Products", "12"),
        ("Page + layout catalog loads", "1"),
        ("Cold / no-change build", "899 / 37 ms"),
        ("Render process", "1.60 ms"),
        ("Throughput at concurrency 50", "16,960 req/s"),
        ("p50 / p95 / p99", "2.66 / 4.43 / 5.70 ms"),
        ("Ordinary route JavaScript", "0 B"),
    ]);
    let scale = table(&[
        ("Routes", "1,000"),
        ("Cold / warm check", "456 / 60 ms"),
        ("Cold / no-change build", "1,628 / 60 ms"),
        ("Shared leaf edit", "973 ms"),
        ("Throughput at concurrency 50", "16,452 req/s"),
        ("p50 / p95 / p99", "2.14 / 6.76 / 13.02 ms"),
        ("RSS after load", "2.80 MB"),
    ]);
    let content = html::fragment()
        .child(section("canonical", "Seven-route application", small))
        .child(section(
            "storefront",
            "Typed-state storefront",
            storefront,
        ))
        .child(section("scale", "1,000 routes and HTTP load", scale))
        .child(section(
            "reading-results",
            "Reading the results",
            html::fragment()
                .child(html::p().text(
                    "Route discovery remains inexpensive as the fixture scales. The slower shared-leaf edit is expected because every generated route module depends on that component; dependency fan-out is now the meaningful compile-time target.",
                ))
                .child(callout(
                    "warning",
                    "Local synthetic measurements",
                    "These numbers are evidence about this implementation on one machine, not universal capacity claims. The load test uses tiny localhost responses and the current thread-per-connection server.",
                )),
        ));
    docs_page(
        "performance",
        "Measured, not marketed",
        "Performance",
        "Build speed, route scale, runtime latency, memory, and the limitations behind every number.",
        content,
    )
}

fn table(rows: &[(&str, &str)]) -> Html {
    let body = rows
        .iter()
        .map(|(measurement, result)| {
            view! {
                tr {
                    td [styles = [u::P_3, u::BORDER_B, u::TEXT_LEFT]] { (*measurement) }
                    td [styles = [u::P_3, u::BORDER_B, u::TEXT_LEFT, u::TEXT_ACCENT, u::FONT_MONO]] { (*result) }
                }
            }
        })
        .collect::<Html>();
    view! {
        table [styles = [u::W_FULL, theme::BORDER_COLLAPSE, theme::TEXT_09]] {
            thead {
                tr {
                    (html::th().attr("scope", "col").styles([u::P_3, u::BORDER_B, u::TEXT_LEFT, u::TEXT_MUTED, theme::TEXT_072, u::UPPERCASE, theme::TRACKING_WIDE]).text("Measurement"))
                    (html::th().attr("scope", "col").styles([u::P_3, u::BORDER_B, u::TEXT_LEFT, u::TEXT_MUTED, theme::TEXT_072, u::UPPERCASE, theme::TRACKING_WIDE]).text("Result"))
                }
            }
            tbody { (body) }
        }
    }
}
