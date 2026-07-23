use crate::components::docs::{callout, docs_page, section};
use crate::components::theme;
use antixt::css::u;
use antixt::{Context, Html, html, view};

pub fn page(_context: Context<'_>) -> Html {
    let small = table(&[
        ("Cold release build", "861 ms"),
        ("No-change release build", "37 ms"),
        ("One-file application edit", "231 ms"),
        ("Render process", "1.76 ms"),
        ("Native server", "558,304 B"),
        ("Ordinary route JavaScript", "0 B"),
    ]);
    let scale = table(&[
        ("Routes", "1,000"),
        ("Cold / warm check", "448 / 61 ms"),
        ("Cold / no-change build", "1,968 / 62 ms"),
        ("Shared leaf edit", "1,048 ms"),
        ("Throughput at concurrency 50", "17,320 req/s"),
        ("p50 / p95 / p99", "2.40 / 4.41 / 8.90 ms"),
        ("RSS after load", "2.85 MB"),
    ]);
    let content = html::fragment()
        .child(section("canonical", "Seven-route application", small))
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
