//! Project design tokens that extend antixt's built-in `css::u` utilities.
//!
//! Keeping these as Rust constants gives rust-analyzer completion and turns a
//! typo into a compiler error while preserving readable utility class output.

use antixt::css::Utility;

macro_rules! utilities {
    ($($rust:ident => ($class:literal, $property:literal, $value:literal)),* $(,)?) => {
        $(pub const $rust: Utility = Utility::named($class, $property, $value);)*
    };
}

utilities! {
    FIXED => ("fixed", "position", "fixed"),
    STATIC => ("static", "position", "static"),
    TOP_0 => ("top-0", "top", "0"),
    TOP_4 => ("top-4", "top", "1rem"),
    TOP_25 => ("top-25", "top", "6.25rem"),
    NEG_TOP_20 => ("-top-20", "top", "-5rem"),
    LEFT_4 => ("left-4", "left", "1rem"),
    Z_20 => ("z-20", "z-index", "20"),
    Z_50 => ("z-50", "z-index", "50"),

    MAX_W_7XL => ("max-w-7xl", "max-width", "1240px"),
    MAX_W_3XL => ("max-w-3xl", "max-width", "760px"),
    MAX_W_2XL => ("max-w-2xl", "max-width", "680px"),
    MAX_W_XL => ("max-w-xl", "max-width", "620px"),
    MAX_W_LG => ("max-w-lg", "max-width", "440px"),
    W_PAGE => ("w-page", "width", "min(1240px,calc(100% - 2rem))"),
    W_8 => ("w-8", "width", "2rem"),
    H_8 => ("h-8", "height", "2rem"),
    MIN_H_12 => ("min-h-12", "min-height", "3rem"),
    MIN_H_17 => ("min-h-17", "min-height", "4.25rem"),
    MIN_H_HERO => ("min-h-hero", "min-height", "700px"),
    MIN_H_CARD => ("min-h-card", "min-height", "230px"),

    GRID_COLS_HERO => ("grid-cols-hero", "grid-template-columns", "minmax(0,1.15fr) minmax(320px,.85fr)"),
    GRID_COLS_DOCS => ("grid-cols-docs", "grid-template-columns", "250px minmax(0,760px)"),
    GAP_FLUID => ("gap-fluid", "gap", "clamp(3rem,8vw,7rem)"),
    GAP_DOCS => ("gap-docs", "gap", "clamp(3rem,8vw,8rem)"),

    TEXT_DISPLAY => ("text-display", "font-size", "clamp(4rem,9vw,8.6rem)"),
    TEXT_HEADING => ("text-heading", "font-size", "clamp(2.5rem,6vw,5rem)"),
    TEXT_DOC_TITLE => ("text-doc-title", "font-size", "clamp(3rem,7vw,5.8rem)"),
    TEXT_2XL => ("text-2xl", "font-size", "1.8rem"),
    TEXT_13 => ("text-13", "font-size", "1.3rem"),
    TEXT_12 => ("text-12", "font-size", "1.2rem"),
    TEXT_09 => ("text-09", "font-size", ".9rem"),
    TEXT_086 => ("text-086", "font-size", ".86rem"),
    TEXT_084 => ("text-084", "font-size", ".84rem"),
    TEXT_08 => ("text-08", "font-size", ".8rem"),
    TEXT_078 => ("text-078", "font-size", ".78rem"),
    TEXT_076 => ("text-076", "font-size", ".76rem"),
    TEXT_074 => ("text-074", "font-size", ".74rem"),
    TEXT_072 => ("text-072", "font-size", ".72rem"),
    TEXT_066 => ("text-066", "font-size", ".66rem"),
    FONT_790 => ("font-790", "font-weight", "790"),
    LEADING_NONE => ("leading-none", "line-height", ".95"),
    LEADING_TIGHT => ("leading-tight", "line-height", ".92"),
    LEADING_DISPLAY => ("leading-display", "line-height", ".84"),
    LEADING_RELAXED => ("leading-relaxed", "line-height", "1.7"),
    LEADING_LOOSE => ("leading-loose", "line-height", "1.75"),
    TRACKING_TIGHT => ("tracking-tight", "letter-spacing", "-.035em"),
    TRACKING_TIGHTER => ("tracking-tighter", "letter-spacing", "-.065em"),
    TRACKING_DISPLAY => ("tracking-display", "letter-spacing", "-.085em"),
    TRACKING_DOC_TITLE => ("tracking-doc-title", "letter-spacing", "-.07em"),
    TRACKING_STAT => ("tracking-stat", "letter-spacing", "-.05em"),
    TRACKING_WIDE => ("tracking-wide", "letter-spacing", ".14em"),
    TRACKING_WIDER => ("tracking-wider", "letter-spacing", ".16em"),
    TRACKING_DOTS => ("tracking-dots", "letter-spacing", ".2em"),
    TRACKING_BRAND => ("tracking-brand", "letter-spacing", "-.03em"),

    BORDER_LINE => ("border-line", "border-color", "var(--line)"),
    BORDER_B_SUBTLE => ("border-b-subtle", "border-bottom", "1px solid rgba(255,255,255,.08)"),
    BORDER_L_2 => ("border-l-2", "border-left-width", "2px"),
    BORDER_L_3 => ("border-l-3", "border-left-width", "3px"),
    BORDER_CYAN => ("border-cyan", "border-color", "var(--cyan)"),
    BORDER_DANGER => ("border-danger", "border-color", "var(--danger)"),
    BORDER_TRANSPARENT => ("border-transparent", "border-color", "transparent"),
    BG_TOPBAR => ("bg-topbar", "background", "rgba(8,11,15,.88)"),
    BG_CODE => ("bg-code", "background", "#090d12"),
    BG_CODE_BAR => ("bg-code-bar", "background", "#0c1117"),
    BG_ACCENT_SOFT => ("bg-accent-soft", "background", "rgba(183,243,107,.07)"),
    BG_CYAN_SOFT => ("bg-cyan-soft", "background", "rgba(114,215,255,.07)"),
    BG_DANGER_SOFT => ("bg-danger-soft", "background", "rgba(255,179,138,.07)"),
    TEXT_INK => ("text-ink", "color", "#11170b"),
    TEXT_BODY => ("text-body", "color", "#bdc7c1"),
    TEXT_CODE => ("text-code", "color", "#d3ddd7"),
    TEXT_CODE_MUTED => ("text-code-muted", "color", "#c7d2cc"),
    TEXT_DOTS => ("text-dots", "color", "#52606d"),
    TEXT_DIM => ("text-dim", "color", "#627068"),

    BACKDROP_BLUR => ("backdrop-blur", "backdrop-filter", "blur(18px)"),
    SHADOW_MARK => ("shadow-mark", "box-shadow", "4px 4px 0 #294510"),
    SHADOW_TERMINAL => ("shadow-terminal", "box-shadow", "18px 22px 0 rgba(183,243,107,.08)"),
    SHADOW_FOCUS => ("shadow-focus", "box-shadow", "0 0 0 3px rgba(183,243,107,.12)"),
    ROTATE_NEG_3 => ("-rotate-3", "transform", "rotate(-3deg)"),
    ROTATE_1 => ("rotate-1", "transform", "rotate(1deg)"),
    TRANSLATE_Y_NEG_05 => ("-translate-y-0_5", "transform", "translateY(-2px)"),
    OUTLINE_NONE => ("outline-none", "outline", "none"),
    CURSOR_POINTER => ("cursor-pointer", "cursor", "pointer")
}

pub const PLACE_ITEMS_CENTER: Utility =
    Utility::named("place-items-center", "place-items", "center");
pub const BORDER_COLLAPSE: Utility =
    Utility::named("border-collapse", "border-collapse", "collapse");
