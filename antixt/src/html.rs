use crate::css::Utility;
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fmt::Display;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Html(Node);

/// Converts ordinary Rust values into escaped HTML nodes.
///
/// `view!` child expressions and [`Html::child`] both use this trait, so
/// components can return a node, optional node, collection, or text value
/// without introducing a second template-language type system.
pub trait IntoHtml {
    fn into_html(self) -> Html;
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Node {
    Element {
        tag: &'static str,
        attributes: Vec<(String, String)>,
        utilities: Vec<Utility>,
        children: Vec<Node>,
        document: bool,
        void: bool,
    },
    Text(String),
    Fragment(Vec<Node>),
}

impl Html {
    pub fn element(tag: &'static str) -> Self {
        Self(Node::Element {
            tag,
            attributes: Vec::new(),
            utilities: Vec::new(),
            children: Vec::new(),
            document: false,
            void: false,
        })
    }

    pub fn document() -> Self {
        Self(Node::Element {
            tag: "html",
            attributes: Vec::new(),
            utilities: Vec::new(),
            children: Vec::new(),
            document: true,
            void: false,
        })
    }

    pub fn void(tag: &'static str) -> Self {
        Self(Node::Element {
            tag,
            attributes: Vec::new(),
            utilities: Vec::new(),
            children: Vec::new(),
            document: false,
            void: true,
        })
    }

    pub fn fragment() -> Self {
        Self(Node::Fragment(Vec::new()))
    }

    pub fn child(mut self, child: impl IntoHtml) -> Self {
        let child = child.into_html();
        match &mut self.0 {
            Node::Element { children, void, .. } => {
                assert!(!*void, "void HTML elements cannot have children");
                children.push(child.0);
            }
            Node::Fragment(children) => children.push(child.0),
            Node::Text(_) => panic!("text nodes cannot have children"),
        }
        self
    }

    pub fn text(self, value: impl Display) -> Self {
        self.child(Self(Node::Text(value.to_string())))
    }

    pub fn attr(mut self, name: impl Into<String>, value: impl Display) -> Self {
        match &mut self.0 {
            Node::Element { attributes, .. } => {
                attributes.push((name.into(), value.to_string()));
            }
            Node::Text(_) | Node::Fragment(_) => panic!("only elements can have attributes"),
        }
        self
    }

    pub fn class(self, value: impl Display) -> Self {
        self.attr("class", value)
    }

    pub fn id(self, value: impl Display) -> Self {
        self.attr("id", value)
    }

    pub fn styles<I>(mut self, styles: I) -> Self
    where
        I: IntoIterator<Item = Utility>,
    {
        match &mut self.0 {
            Node::Element { utilities, .. } => utilities.extend(styles),
            Node::Text(_) | Node::Fragment(_) => panic!("only elements can have styles"),
        }
        self
    }

    pub fn fragment_get(self, url: impl Display) -> Self {
        self.attr("data-antixt-get", url)
    }

    pub fn fragment_post(self, url: impl Display) -> Self {
        self.attr("data-antixt-post", url)
    }

    pub fn fragment_target(self, selector: impl Display) -> Self {
        self.attr("data-antixt-target", selector)
    }

    pub fn fragment_swap(self, strategy: impl Display) -> Self {
        self.attr("data-antixt-swap", strategy)
    }

    pub fn fragment_form(self) -> Self {
        self.attr("data-antixt-fragment", "")
    }

    pub fn island(self, module: impl Display) -> Self {
        self.attr("data-antixt-island", module)
    }

    pub fn render(&self) -> String {
        let stylesheet = stylesheet(&self.0);
        let mut output = String::new();
        let is_document = matches!(self.0, Node::Element { document: true, .. });
        if !is_document && !stylesheet.is_empty() {
            render_style(&stylesheet, &mut output);
        }
        render_node(
            &self.0,
            &mut output,
            is_document.then_some(stylesheet.as_str()),
        );
        output
    }
}

impl IntoHtml for Html {
    fn into_html(self) -> Html {
        self
    }
}

impl IntoHtml for String {
    fn into_html(self) -> Html {
        Html(Node::Text(self))
    }
}

impl IntoHtml for &str {
    fn into_html(self) -> Html {
        Html(Node::Text(self.to_owned()))
    }
}

impl IntoHtml for &String {
    fn into_html(self) -> Html {
        Html(Node::Text(self.clone()))
    }
}

macro_rules! display_into_html {
    ($($type:ty),* $(,)?) => {
        $(
            impl IntoHtml for $type {
                fn into_html(self) -> Html {
                    Html(Node::Text(self.to_string()))
                }
            }
        )*
    };
}

display_into_html!(
    bool, char, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64,
);

impl IntoHtml for Cow<'_, str> {
    fn into_html(self) -> Html {
        Html(Node::Text(self.into_owned()))
    }
}

impl<T: IntoHtml> IntoHtml for Option<T> {
    fn into_html(self) -> Html {
        self.map_or_else(Html::fragment, IntoHtml::into_html)
    }
}

impl<T: IntoHtml> IntoHtml for Vec<T> {
    fn into_html(self) -> Html {
        self.into_iter().collect()
    }
}

impl<T: IntoHtml, const N: usize> IntoHtml for [T; N] {
    fn into_html(self) -> Html {
        self.into_iter().collect()
    }
}

impl<T: IntoHtml> FromIterator<T> for Html {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        iter.into_iter()
            .fold(Html::fragment(), |fragment, child| fragment.child(child))
    }
}

fn render_node(node: &Node, output: &mut String, stylesheet: Option<&str>) {
    match node {
        Node::Text(value) => escape(value, output),
        Node::Fragment(children) => {
            for child in children {
                render_node(child, output, stylesheet);
            }
        }
        Node::Element {
            tag,
            attributes,
            utilities,
            children,
            document,
            void,
        } => {
            if *document {
                output.push_str("<!doctype html>");
            }
            output.push('<');
            output.push_str(tag);
            let mut classes = attributes
                .iter()
                .filter(|(name, _)| name == "class")
                .map(|(_, value)| value.as_str())
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            for utility in utilities {
                if !classes.is_empty() {
                    classes.push(' ');
                }
                classes.push_str(&utility.class_name());
            }
            let mut rendered_classes = false;
            for (name, value) in attributes {
                if name == "class" {
                    if rendered_classes {
                        continue;
                    }
                    rendered_classes = true;
                    if classes.is_empty() {
                        continue;
                    }
                    output.push_str(" class=\"");
                    escape(&classes, output);
                    output.push('"');
                    continue;
                }
                output.push(' ');
                output.push_str(name);
                output.push_str("=\"");
                escape(value, output);
                output.push('"');
            }
            if !rendered_classes && !classes.is_empty() {
                output.push_str(" class=\"");
                escape(&classes, output);
                output.push('"');
            }
            output.push('>');
            if !*void {
                for child in children {
                    render_node(child, output, stylesheet);
                }
                if *tag == "head"
                    && let Some(stylesheet) = stylesheet
                    && !stylesheet.is_empty()
                {
                    render_style(stylesheet, output);
                }
                output.push_str("</");
                output.push_str(tag);
                output.push('>');
            }
        }
    }
}

fn stylesheet(node: &Node) -> String {
    let mut seen = BTreeSet::new();
    let mut rules = Vec::new();
    collect_rules(node, &mut seen, &mut rules);
    rules.join("")
}

fn collect_rules(node: &Node, seen: &mut BTreeSet<String>, rules: &mut Vec<String>) {
    match node {
        Node::Element {
            utilities,
            children,
            ..
        } => {
            for utility in utilities {
                let class = utility.class_name();
                if seen.insert(class) {
                    rules.push(utility.rule());
                }
            }
            for child in children {
                collect_rules(child, seen, rules);
            }
        }
        Node::Fragment(children) => {
            for child in children {
                collect_rules(child, seen, rules);
            }
        }
        Node::Text(_) => {}
    }
}

fn render_style(stylesheet: &str, output: &mut String) {
    output.push_str("<style data-antixt-css>");
    output.push_str(stylesheet);
    output.push_str("</style>");
}

fn escape(value: &str, output: &mut String) {
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            _ => output.push(character),
        }
    }
}

pub fn document() -> Html {
    Html::document()
}

pub fn fragment() -> Html {
    Html::fragment()
}

macro_rules! elements {
    ($($name:ident => $tag:literal),* $(,)?) => {
        $(pub fn $name() -> Html { Html::element($tag) })*
    };
}

macro_rules! void_elements {
    ($($name:ident => $tag:literal),* $(,)?) => {
        $(pub fn $name() -> Html { Html::void($tag) })*
    };
}

elements! {
    head => "head",
    body => "body",
    main => "main",
    nav => "nav",
    section => "section",
    article => "article",
    div => "div",
    span => "span",
    h1 => "h1",
    h2 => "h2",
    h3 => "h3",
    h4 => "h4",
    h5 => "h5",
    h6 => "h6",
    p => "p",
    a => "a",
    form => "form",
    button => "button",
    title => "title",
    style => "style",
    header => "header",
    footer => "footer",
    ul => "ul",
    ol => "ol",
    li => "li",
    strong => "strong",
    code => "code",
    pre => "pre",
    label => "label",
    textarea => "textarea",
    aside => "aside",
    blockquote => "blockquote",
    small => "small",
    kbd => "kbd",
    table => "table",
    thead => "thead",
    tbody => "tbody",
    tr => "tr",
    th => "th",
    td => "td",
}

void_elements! {
    meta => "meta",
    input => "input",
    link => "link",
}

#[doc(hidden)]
#[macro_export]
macro_rules! __antixt_view_attributes {
    ($node:ident;) => { $node };
    ($node:ident; styles = $value:expr, $($rest:tt)*) => {{
        let $node = $node.styles($value);
        $crate::__antixt_view_attributes!($node; $($rest)*)
    }};
    ($node:ident; class = $value:expr, $($rest:tt)*) => {{
        let $node = $node.class($value);
        $crate::__antixt_view_attributes!($node; $($rest)*)
    }};
    ($node:ident; id = $value:expr, $($rest:tt)*) => {{
        let $node = $node.id($value);
        $crate::__antixt_view_attributes!($node; $($rest)*)
    }};
    ($node:ident; for_ = $value:expr, $($rest:tt)*) => {{
        let $node = $node.attr("for", $value);
        $crate::__antixt_view_attributes!($node; $($rest)*)
    }};
    ($node:ident; aria_label = $value:expr, $($rest:tt)*) => {{
        let $node = $node.attr("aria-label", $value);
        $crate::__antixt_view_attributes!($node; $($rest)*)
    }};
    ($node:ident; aria_current = $value:expr, $($rest:tt)*) => {{
        let $node = $node.attr("aria-current", $value);
        $crate::__antixt_view_attributes!($node; $($rest)*)
    }};
    ($node:ident; aria_live = $value:expr, $($rest:tt)*) => {{
        let $node = $node.attr("aria-live", $value);
        $crate::__antixt_view_attributes!($node; $($rest)*)
    }};
    ($node:ident; data_doc_title = $value:expr, $($rest:tt)*) => {{
        let $node = $node.attr("data-doc-title", $value);
        $crate::__antixt_view_attributes!($node; $($rest)*)
    }};
    ($node:ident; href = $value:expr, $($rest:tt)*) => {{
        let $node = $node.attr("href", $value);
        $crate::__antixt_view_attributes!($node; $($rest)*)
    }};
    ($node:ident; type_ = $value:expr, $($rest:tt)*) => {{
        let $node = $node.attr("type", $value);
        $crate::__antixt_view_attributes!($node; $($rest)*)
    }};
    ($node:ident; name = $value:expr, $($rest:tt)*) => {{
        let $node = $node.attr("name", $value);
        $crate::__antixt_view_attributes!($node; $($rest)*)
    }};
    ($node:ident; method = $value:expr, $($rest:tt)*) => {{
        let $node = $node.attr("method", $value);
        $crate::__antixt_view_attributes!($node; $($rest)*)
    }};
    ($node:ident; action = $value:expr, $($rest:tt)*) => {{
        let $node = $node.attr("action", $value);
        $crate::__antixt_view_attributes!($node; $($rest)*)
    }};
    ($node:ident; placeholder = $value:expr, $($rest:tt)*) => {{
        let $node = $node.attr("placeholder", $value);
        $crate::__antixt_view_attributes!($node; $($rest)*)
    }};
    ($node:ident; autocomplete = $value:expr, $($rest:tt)*) => {{
        let $node = $node.attr("autocomplete", $value);
        $crate::__antixt_view_attributes!($node; $($rest)*)
    }};
    ($node:ident; title = $value:expr, $($rest:tt)*) => {{
        let $node = $node.attr("title", $value);
        $crate::__antixt_view_attributes!($node; $($rest)*)
    }};
    ($node:ident; role = $value:expr, $($rest:tt)*) => {{
        let $node = $node.attr("role", $value);
        $crate::__antixt_view_attributes!($node; $($rest)*)
    }};
    ($node:ident; lang = $value:expr, $($rest:tt)*) => {{
        let $node = $node.attr("lang", $value);
        $crate::__antixt_view_attributes!($node; $($rest)*)
    }};
    ($node:ident; charset = $value:expr, $($rest:tt)*) => {{
        let $node = $node.attr("charset", $value);
        $crate::__antixt_view_attributes!($node; $($rest)*)
    }};
    ($node:ident; content = $value:expr, $($rest:tt)*) => {{
        let $node = $node.attr("content", $value);
        $crate::__antixt_view_attributes!($node; $($rest)*)
    }};
    ($node:ident; rel = $value:expr, $($rest:tt)*) => {{
        let $node = $node.attr("rel", $value);
        $crate::__antixt_view_attributes!($node; $($rest)*)
    }};
    ($node:ident; crossorigin = $value:expr, $($rest:tt)*) => {{
        let $node = $node.attr("crossorigin", $value);
        $crate::__antixt_view_attributes!($node; $($rest)*)
    }};
    ($node:ident; $attribute:ident = $value:expr, $($rest:tt)*) => {{
        compile_error!(concat!(
            "unsupported typed view! attribute `",
            stringify!($attribute),
            "`; use the builder .attr(...) escape hatch"
        ));
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __antixt_view_children {
    ($node:ident;) => { $node };
    ($node:ident; $tag:ident [$($attribute:ident = $value:expr),* $(,)?] { $($inner:tt)* } $($rest:tt)*) => {{
        let $node = $node.child($crate::view!($tag [$($attribute = $value),*] { $($inner)* }));
        $crate::__antixt_view_children!($node; $($rest)*)
    }};
    ($node:ident; $tag:ident { $($inner:tt)* } $($rest:tt)*) => {{
        let $node = $node.child($crate::view!($tag { $($inner)* }));
        $crate::__antixt_view_children!($node; $($rest)*)
    }};
    ($node:ident; text($value:expr) $($rest:tt)*) => {{
        let $node = $node.text($value);
        $crate::__antixt_view_children!($node; $($rest)*)
    }};
    ($node:ident; ($child:expr) $($rest:tt)*) => {{
        let $node = $node.child($child);
        $crate::__antixt_view_children!($node; $($rest)*)
    }};
    ($node:ident; $text:literal $($rest:tt)*) => {{
        let $node = $node.text($text);
        $crate::__antixt_view_children!($node; $($rest)*)
    }};
}

#[macro_export]
macro_rules! view {
    ($tag:ident [$($attribute:ident = $value:expr),* $(,)?] { $($children:tt)* }) => {{
        let node = $crate::html::$tag();
        let node = $crate::__antixt_view_attributes!(node; $($attribute = $value,)*);
        $crate::__antixt_view_children!(node; $($children)*)
    }};
    ($tag:ident { $($children:tt)* }) => {{
        let node = $crate::html::$tag();
        $crate::__antixt_view_children!(node; $($children)*)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_typed_elements_and_escapes_values() {
        let page = main()
            .class("shell")
            .child(h1().text("A < B"))
            .child(p().text("safe & sound"));
        assert_eq!(
            page.render(),
            "<main class=\"shell\"><h1>A &lt; B</h1><p>safe &amp; sound</p></main>"
        );
    }

    #[test]
    fn renders_a_document_doctype() {
        let page = document().attr("lang", "en").child(body());
        assert_eq!(
            page.render(),
            "<!doctype html><html lang=\"en\"><body></body></html>"
        );
    }

    #[test]
    fn renders_nested_view_macro_with_rust_expressions() {
        let heading = "Typed & terse";
        let page = crate::view! {
            main [class = "shell"] {
                h1 { text(heading) }
                p { "Normal Rust macro syntax." }
                (strong().text("component"))
            }
        };
        assert_eq!(
            page.render(),
            "<main class=\"shell\"><h1>Typed &amp; terse</h1><p>Normal Rust macro syntax.</p><strong>component</strong></main>"
        );
    }

    #[test]
    fn view_macro_accepts_typed_styles_optional_children_and_collections() {
        use crate::css::{self, Display, Space};

        let show_note = true;
        let links = ["Docs", "Benchmarks"]
            .into_iter()
            .map(|label| crate::view! { a [href = "/"] { (label) } })
            .collect::<Html>();
        let page = crate::view! {
            main [
                class = "shell",
                aria_label = "Example",
                styles = [css::display(Display::Flex), css::gap(Space::Three)],
            ] {
                (show_note.then(|| crate::view! { p { "Ready" } }))
                (links)
            }
        };
        let output = page.render();

        assert!(output.contains("<main class=\"shell f-"));
        assert!(output.contains("aria-label=\"Example\""));
        assert!(output.contains("<p>Ready</p><a href=\"/\">Docs</a>"));
        assert!(output.contains("{display:flex}"));
        assert!(output.contains("{gap:.75rem}"));
    }

    #[test]
    fn into_html_text_values_are_escaped() {
        let children = vec![String::from("A < B"), String::from("safe & sound")];
        assert_eq!(
            div().child(children).render(),
            "<div>A &lt; Bsafe &amp; sound</div>"
        );
        assert_eq!(span().child(42).render(), "<span>42</span>");
    }

    #[test]
    fn injects_deduplicated_typed_utility_css_into_documents() {
        use crate::css::{self, Breakpoint, Color, Display, Space};

        let utility = css::display(Display::Flex);
        let page = document()
            .child(head().child(style().text("body{margin:0}")))
            .child(
                body()
                    .child(div().styles([utility.clone(), css::gap(Space::Four)]))
                    .child(div().styles([
                        utility,
                        css::at(
                            Breakpoint::Medium,
                            css::hover(css::background(Color::Raised)),
                        ),
                    ])),
            );
        let output = page.render();
        assert_eq!(output.matches("{display:flex}").count(), 1);
        assert!(output.contains("data-antixt-css"));
        assert!(output.contains("@media (min-width: 48rem)"));
        assert!(output.find("body{margin:0}").unwrap() < output.find("data-antixt-css").unwrap());
    }

    #[test]
    fn includes_typed_utility_css_with_html_fragments() {
        use crate::css::{self, Color};

        let output = p()
            .styles([css::text_color(Color::Accent)])
            .text("Fragment")
            .render();
        assert!(output.starts_with("<style data-antixt-css>"));
        assert!(output.contains("{color:var(--accent)}"));
    }
}
