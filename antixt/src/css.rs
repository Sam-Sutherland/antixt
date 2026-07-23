//! Typed, atomic CSS utilities generated from ordinary Rust values.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Utility {
    name: Option<&'static str>,
    property: &'static str,
    value: &'static str,
    pseudo: Option<&'static str>,
    pseudo_name: Option<&'static str>,
    media: Option<&'static str>,
    media_name: Option<&'static str>,
}

impl Utility {
    const fn new(property: &'static str, value: &'static str) -> Self {
        Self {
            name: None,
            property,
            value,
            pseudo: None,
            pseudo_name: None,
            media: None,
            media_name: None,
        }
    }

    /// Defines a project utility with a stable, readable class name.
    ///
    /// Prefer the built-ins in [`u`]. This is the configuration escape hatch
    /// for project tokens such as `grid-cols-docs`.
    pub const fn named(name: &'static str, property: &'static str, value: &'static str) -> Self {
        Self {
            name: Some(name),
            property,
            value,
            pseudo: None,
            pseudo_name: None,
            media: None,
            media_name: None,
        }
    }

    pub(crate) fn class_name(&self) -> String {
        let base = self
            .name
            .map(str::to_owned)
            .unwrap_or_else(|| format!("f-{:016x}", hash(self.key().as_bytes())));
        [self.pseudo_name, self.media_name]
            .into_iter()
            .flatten()
            .fold(base, |class, variant| format!("{variant}:{class}"))
    }

    pub(crate) fn rule(&self) -> String {
        let class = self.class_name();
        let pseudo = self.pseudo.unwrap_or("");
        let selector = class.replace(':', "\\:");
        let declaration = format!(".{selector}{pseudo}{{{}:{}}}", self.property, self.value);
        if let Some(media) = self.media {
            format!("@media {media}{{{declaration}}}")
        } else {
            declaration
        }
    }

    fn key(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.media.unwrap_or("base"),
            self.pseudo.unwrap_or("base"),
            self.property,
            self.value
        )
    }
}

const fn hash(bytes: &[u8]) -> u64 {
    let mut value = 0xcbf29ce484222325_u64;
    let mut index = 0;
    while index < bytes.len() {
        value ^= bytes[index] as u64;
        value = value.wrapping_mul(0x100000001b3);
        index += 1;
    }
    value
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Display {
    Block,
    InlineBlock,
    Flex,
    Grid,
    None,
}

impl Display {
    const fn value(self) -> &'static str {
        match self {
            Self::Block => "block",
            Self::InlineBlock => "inline-block",
            Self::Flex => "flex",
            Self::Grid => "grid",
            Self::None => "none",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Space {
    Zero,
    One,
    Two,
    Three,
    Four,
    Six,
    Eight,
    Twelve,
    Sixteen,
    TwentyFour,
}

impl Space {
    const fn value(self) -> &'static str {
        match self {
            Self::Zero => "0",
            Self::One => ".25rem",
            Self::Two => ".5rem",
            Self::Three => ".75rem",
            Self::Four => "1rem",
            Self::Six => "1.5rem",
            Self::Eight => "2rem",
            Self::Twelve => "3rem",
            Self::Sixteen => "4rem",
            Self::TwentyFour => "6rem",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Color {
    Background,
    Panel,
    Raised,
    Text,
    Muted,
    Accent,
    Cyan,
    Transparent,
    Current,
}

impl Color {
    const fn value(self) -> &'static str {
        match self {
            Self::Background => "var(--bg)",
            Self::Panel => "var(--panel)",
            Self::Raised => "var(--raised)",
            Self::Text => "var(--text)",
            Self::Muted => "var(--muted)",
            Self::Accent => "var(--accent)",
            Self::Cyan => "var(--cyan)",
            Self::Transparent => "transparent",
            Self::Current => "currentColor",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Radius {
    None,
    Small,
    Medium,
    Large,
    Full,
}

impl Radius {
    const fn value(self) -> &'static str {
        match self {
            Self::None => "0",
            Self::Small => ".25rem",
            Self::Medium => ".5rem",
            Self::Large => ".8rem",
            Self::Full => "9999px",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontSize {
    ExtraSmall,
    Small,
    Base,
    Large,
    ExtraLarge,
    Display,
}

impl FontSize {
    const fn value(self) -> &'static str {
        match self {
            Self::ExtraSmall => ".75rem",
            Self::Small => ".875rem",
            Self::Base => "1rem",
            Self::Large => "1.125rem",
            Self::ExtraLarge => "1.5rem",
            Self::Display => "clamp(3rem, 8vw, 7rem)",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Weight {
    Normal,
    Medium,
    Semibold,
    Bold,
    Black,
}

impl Weight {
    const fn value(self) -> &'static str {
        match self {
            Self::Normal => "400",
            Self::Medium => "500",
            Self::Semibold => "600",
            Self::Bold => "700",
            Self::Black => "800",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}

impl Align {
    const fn value(self) -> &'static str {
        match self {
            Self::Start => "flex-start",
            Self::Center => "center",
            Self::End => "flex-end",
            Self::Stretch => "stretch",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Justify {
    Start,
    Center,
    End,
    Between,
}

impl Justify {
    const fn value(self) -> &'static str {
        match self {
            Self::Start => "flex-start",
            Self::Center => "center",
            Self::End => "flex-end",
            Self::Between => "space-between",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Wrap {
    NoWrap,
    Wrap,
}

impl Wrap {
    const fn value(self) -> &'static str {
        match self {
            Self::NoWrap => "nowrap",
            Self::Wrap => "wrap",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GridColumns {
    One,
    Two,
    Three,
    Four,
}

impl GridColumns {
    const fn value(self) -> &'static str {
        match self {
            Self::One => "repeat(1,minmax(0,1fr))",
            Self::Two => "repeat(2,minmax(0,1fr))",
            Self::Three => "repeat(3,minmax(0,1fr))",
            Self::Four => "repeat(4,minmax(0,1fr))",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Breakpoint {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

impl Breakpoint {
    const fn value(self) -> &'static str {
        match self {
            Self::Small => "(min-width: 40rem)",
            Self::Medium => "(min-width: 48rem)",
            Self::Large => "(min-width: 64rem)",
            Self::ExtraLarge => "(min-width: 80rem)",
        }
    }
}

pub const fn display(value: Display) -> Utility {
    Utility::new("display", value.value())
}

pub const fn gap(value: Space) -> Utility {
    Utility::new("gap", value.value())
}

pub const fn padding(value: Space) -> Utility {
    Utility::new("padding", value.value())
}

pub const fn padding_x(value: Space) -> Utility {
    Utility::new("padding-inline", value.value())
}

pub const fn padding_y(value: Space) -> Utility {
    Utility::new("padding-block", value.value())
}

pub const fn margin_top(value: Space) -> Utility {
    Utility::new("margin-top", value.value())
}

pub const fn background(value: Color) -> Utility {
    Utility::new("background", value.value())
}

pub const fn text_color(value: Color) -> Utility {
    Utility::new("color", value.value())
}

pub const fn radius(value: Radius) -> Utility {
    Utility::new("border-radius", value.value())
}

pub const fn font_size(value: FontSize) -> Utility {
    Utility::new("font-size", value.value())
}

pub const fn font_weight(value: Weight) -> Utility {
    Utility::new("font-weight", value.value())
}

pub const fn align_items(value: Align) -> Utility {
    Utility::new("align-items", value.value())
}

pub const fn justify_content(value: Justify) -> Utility {
    Utility::new("justify-content", value.value())
}

pub const fn flex_wrap(value: Wrap) -> Utility {
    Utility::new("flex-wrap", value.value())
}

pub const fn grid_columns(value: GridColumns) -> Utility {
    Utility::new("grid-template-columns", value.value())
}

pub const fn width_full() -> Utility {
    Utility::new("width", "100%")
}

pub const fn hover(mut utility: Utility) -> Utility {
    utility.pseudo = Some(":hover");
    utility.pseudo_name = Some("hover");
    utility
}

pub const fn focus_visible(mut utility: Utility) -> Utility {
    utility.pseudo = Some(":focus-visible");
    utility.pseudo_name = Some("focus-visible");
    utility
}

pub const fn at(breakpoint: Breakpoint, mut utility: Utility) -> Utility {
    utility.media = Some(breakpoint.value());
    utility.media_name = Some(match breakpoint {
        Breakpoint::Small => "sm",
        Breakpoint::Medium => "md",
        Breakpoint::Large => "lg",
        Breakpoint::ExtraLarge => "xl",
    });
    utility
}

/// Autocomplete-friendly Tailwind-style utilities.
///
/// Rust uses underscores in identifiers, while rendered HTML uses familiar
/// kebab-case names: `u::P_2` becomes `p-2` and `u::ITEMS_CENTER` becomes
/// `items-center`.
pub mod u {
    use super::Utility;

    macro_rules! utilities {
        ($($rust:ident => ($class:literal, $property:literal, $value:literal)),* $(,)?) => {
            $(pub const $rust: Utility = Utility::named($class, $property, $value);)*
        };
    }

    utilities! {
        BLOCK => ("block", "display", "block"),
        INLINE_BLOCK => ("inline-block", "display", "inline-block"),
        INLINE_FLEX => ("inline-flex", "display", "inline-flex"),
        FLEX => ("flex", "display", "flex"),
        FLEX_ROW => ("flex-row", "flex-direction", "row"),
        FLEX_COL => ("flex-col", "flex-direction", "column"),
        GRID => ("grid", "display", "grid"),
        HIDDEN => ("hidden", "display", "none"),
        RELATIVE => ("relative", "position", "relative"),
        STICKY => ("sticky", "position", "sticky"),
        W_FULL => ("w-full", "width", "100%"),
        MIN_W_0 => ("min-w-0", "min-width", "0"),
        OVERFLOW_HIDDEN => ("overflow-hidden", "overflow", "hidden"),
        OVERFLOW_X_AUTO => ("overflow-x-auto", "overflow-x", "auto"),

        P_0 => ("p-0", "padding", "0"),
        P_1 => ("p-1", "padding", ".25rem"),
        P_2 => ("p-2", "padding", ".5rem"),
        P_3 => ("p-3", "padding", ".75rem"),
        P_4 => ("p-4", "padding", "1rem"),
        P_6 => ("p-6", "padding", "1.5rem"),
        P_8 => ("p-8", "padding", "2rem"),
        P_12 => ("p-12", "padding", "3rem"),
        PX_2 => ("px-2", "padding-inline", ".5rem"),
        PX_3 => ("px-3", "padding-inline", ".75rem"),
        PX_4 => ("px-4", "padding-inline", "1rem"),
        PY_1 => ("py-1", "padding-block", ".25rem"),
        PY_2 => ("py-2", "padding-block", ".5rem"),
        PY_3 => ("py-3", "padding-block", ".75rem"),
        PY_4 => ("py-4", "padding-block", "1rem"),
        PY_8 => ("py-8", "padding-block", "2rem"),
        PY_12 => ("py-12", "padding-block", "3rem"),
        PY_16 => ("py-16", "padding-block", "4rem"),
        PY_24 => ("py-24", "padding-block", "6rem"),
        PB_12 => ("pb-12", "padding-bottom", "3rem"),

        M_0 => ("m-0", "margin", "0"),
        M_4 => ("m-4", "margin", "1rem"),
        MY_6 => ("my-6", "margin-block", "1.5rem"),
        MX_AUTO => ("mx-auto", "margin-inline", "auto"),
        MT_0 => ("mt-0", "margin-top", "0"),
        MT_1 => ("mt-1", "margin-top", ".25rem"),
        MT_2 => ("mt-2", "margin-top", ".5rem"),
        MT_3 => ("mt-3", "margin-top", ".75rem"),
        MT_4 => ("mt-4", "margin-top", "1rem"),
        MT_6 => ("mt-6", "margin-top", "1.5rem"),
        MT_8 => ("mt-8", "margin-top", "2rem"),
        MB_0 => ("mb-0", "margin-bottom", "0"),
        MB_1 => ("mb-1", "margin-bottom", ".25rem"),
        MB_2 => ("mb-2", "margin-bottom", ".5rem"),
        MB_3 => ("mb-3", "margin-bottom", ".75rem"),
        MB_4 => ("mb-4", "margin-bottom", "1rem"),
        MB_6 => ("mb-6", "margin-bottom", "1.5rem"),
        MB_8 => ("mb-8", "margin-bottom", "2rem"),
        MB_12 => ("mb-12", "margin-bottom", "3rem"),

        GAP_0 => ("gap-0", "gap", "0"),
        GAP_1 => ("gap-1", "gap", ".25rem"),
        GAP_2 => ("gap-2", "gap", ".5rem"),
        GAP_3 => ("gap-3", "gap", ".75rem"),
        GAP_4 => ("gap-4", "gap", "1rem"),
        GAP_6 => ("gap-6", "gap", "1.5rem"),
        GAP_8 => ("gap-8", "gap", "2rem"),
        GAP_12 => ("gap-12", "gap", "3rem"),

        ITEMS_START => ("items-start", "align-items", "flex-start"),
        ITEMS_CENTER => ("items-center", "align-items", "center"),
        ITEMS_END => ("items-end", "align-items", "flex-end"),
        ITEMS_STRETCH => ("items-stretch", "align-items", "stretch"),
        JUSTIFY_START => ("justify-start", "justify-content", "flex-start"),
        JUSTIFY_CENTER => ("justify-center", "justify-content", "center"),
        JUSTIFY_END => ("justify-end", "justify-content", "flex-end"),
        JUSTIFY_BETWEEN => ("justify-between", "justify-content", "space-between"),
        FLEX_WRAP => ("flex-wrap", "flex-wrap", "wrap"),
        FLEX_NOWRAP => ("flex-nowrap", "flex-wrap", "nowrap"),
        GRID_COLS_1 => ("grid-cols-1", "grid-template-columns", "repeat(1,minmax(0,1fr))"),
        GRID_COLS_2 => ("grid-cols-2", "grid-template-columns", "repeat(2,minmax(0,1fr))"),
        GRID_COLS_3 => ("grid-cols-3", "grid-template-columns", "repeat(3,minmax(0,1fr))"),
        GRID_COLS_4 => ("grid-cols-4", "grid-template-columns", "repeat(4,minmax(0,1fr))"),

        BG_BACKGROUND => ("bg-background", "background", "var(--bg)"),
        BG_PANEL => ("bg-panel", "background", "var(--panel)"),
        BG_RAISED => ("bg-raised", "background", "var(--raised)"),
        BG_ACCENT => ("bg-accent", "background", "var(--accent)"),
        BG_TRANSPARENT => ("bg-transparent", "background", "transparent"),
        TEXT_DEFAULT => ("text-default", "color", "var(--text)"),
        TEXT_MUTED => ("text-muted", "color", "var(--muted)"),
        TEXT_ACCENT => ("text-accent", "color", "var(--accent)"),
        TEXT_CYAN => ("text-cyan", "color", "var(--cyan)"),

        TEXT_XS => ("text-xs", "font-size", ".75rem"),
        TEXT_SM => ("text-sm", "font-size", ".875rem"),
        TEXT_BASE => ("text-base", "font-size", "1rem"),
        TEXT_LG => ("text-lg", "font-size", "1.125rem"),
        TEXT_XL => ("text-xl", "font-size", "1.5rem"),
        FONT_NORMAL => ("font-normal", "font-weight", "400"),
        FONT_MEDIUM => ("font-medium", "font-weight", "500"),
        FONT_SEMIBOLD => ("font-semibold", "font-weight", "600"),
        FONT_BOLD => ("font-bold", "font-weight", "700"),
        FONT_BLACK => ("font-black", "font-weight", "800"),
        FONT_MONO => ("font-mono", "font-family", "var(--font-mono)"),

        ROUNDED_NONE => ("rounded-none", "border-radius", "0"),
        ROUNDED_SM => ("rounded-sm", "border-radius", ".25rem"),
        ROUNDED => ("rounded", "border-radius", ".5rem"),
        ROUNDED_LG => ("rounded-lg", "border-radius", ".8rem"),
        ROUNDED_FULL => ("rounded-full", "border-radius", "9999px"),
        BORDER => ("border", "border", "1px solid var(--line)"),
        BORDER_T => ("border-t", "border-top", "1px solid var(--line)"),
        BORDER_B => ("border-b", "border-bottom", "1px solid var(--line)"),
        BORDER_L => ("border-l", "border-left", "1px solid var(--line)"),
        BORDER_R => ("border-r", "border-right", "1px solid var(--line)"),
        BORDER_ACCENT => ("border-accent", "border-color", "var(--accent)"),

        NO_UNDERLINE => ("no-underline", "text-decoration", "none"),
        TEXT_LEFT => ("text-left", "text-align", "left"),
        UPPERCASE => ("uppercase", "text-transform", "uppercase")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_stable_atomic_rules_and_variants() {
        let base = display(Display::Grid);
        assert_eq!(base.class_name(), display(Display::Grid).class_name());
        assert!(base.rule().contains("{display:grid}"));

        let responsive = at(Breakpoint::Medium, hover(background(Color::Raised)));
        assert!(responsive.rule().starts_with("@media (min-width: 48rem)"));
        assert!(
            responsive
                .rule()
                .contains(":hover{background:var(--raised)}")
        );
    }

    #[test]
    fn named_utilities_render_tailwind_style_classes() {
        assert_eq!(u::P_2.class_name(), "p-2");
        assert_eq!(u::M_4.rule(), ".m-4{margin:1rem}");

        let responsive = at(Breakpoint::Medium, hover(u::BG_RAISED));
        assert_eq!(responsive.class_name(), "md:hover:bg-raised");
        assert!(responsive.rule().contains(".md\\:hover\\:bg-raised:hover"));
    }
}
