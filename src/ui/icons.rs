//! Icon rendering (Tabler Icons + a few hand-drawn Termius-specific icons).

use crate::ui::theme;
use egui::{Align2, Color32, FontFamily, FontId, Painter, Pos2, Stroke, pos2};

#[derive(Clone, Copy)]
pub enum IconStyle {
    Regular,
    Fill,
    Bold,
}

pub fn font(size: f32, style: IconStyle) -> FontId {
    let family = match style {
        IconStyle::Regular | IconStyle::Bold => theme::TI_REGULAR,
        IconStyle::Fill => theme::TI_FILLED,
    };
    FontId::new(size, FontFamily::Name(family.into()))
}

/// Paints a Tabler icon glyph centered on `center`.
pub fn paint(
    painter: &Painter,
    center: Pos2,
    glyph: &str,
    size: f32,
    color: Color32,
    style: IconStyle,
) {
    painter.text(
        center,
        Align2::CENTER_CENTER,
        glyph,
        font(size, style),
        color,
    );
}

/// Tabler Icons (outline, 2px stroke).
#[allow(dead_code)]
pub mod regular {
    pub const ARROW_LINE_RIGHT: &str = "\u{ea0f}"; // arrow-bar-right
    pub const ARROW_SQUARE_OUT: &str = "\u{ea99}"; // external-link
    pub const BELL: &str = "\u{ea35}"; // bell
    pub const BRACKETS_CURLY: &str = "\u{ebcc}"; // braces
    pub const CALENDAR: &str = "\u{ea53}"; // calendar
    pub const CARET_DOWN: &str = "\u{ea5f}"; // chevron-down
    pub const CHECK: &str = "\u{ea5e}"; // check
    pub const CLOCK: &str = "\u{ea70}"; // clock
    pub const CLOUD: &str = "\u{ea76}"; // cloud
    pub const COPY: &str = "\u{ea7a}"; // copy
    pub const EYE: &str = "\u{ea9a}"; // eye
    pub const EYE_SLASH: &str = "\u{ecf0}"; // eye-off
    pub const FOLDER_SIMPLE: &str = "\u{eaad}"; // folder
    pub const GEAR: &str = "\u{eb20}"; // settings
    pub const GLOBE: &str = "\u{eb54}"; // world
    pub const HARD_DRIVES: &str = "\u{eb1f}"; // server
    pub const KEY: &str = "\u{eac7}"; // key
    pub const LINUX_LOGO: &str = "\u{ef59}"; // brand-ubuntu
    pub const LIST: &str = "\u{ec42}"; // menu-2
    pub const LIST_BULLETS: &str = "\u{eb6b}"; // list
    pub const LOCK: &str = "\u{eae2}"; // lock
    pub const MINUS: &str = "\u{eaf2}"; // minus
    pub const PENCIL_SIMPLE: &str = "\u{eb04}"; // pencil
    pub const PLUG: &str = "\u{ebd9}"; // plug
    pub const PLUS: &str = "\u{eb0b}"; // plus
    pub const SHARE_FAT: &str = "\u{eb21}"; // share
    pub const SQUARE: &str = "\u{eb2c}"; // square
    pub const SQUARES_FOUR: &str = "\u{edba}"; // layout-grid
    pub const TAG: &str = "\u{10096}"; // tag
    pub const TERMINAL_WINDOW: &str = "\u{ebef}"; // terminal-2
    pub const USER: &str = "\u{eb4d}"; // user
    pub const VAULT: &str = "\u{ed58}"; // shield-lock
    pub const X: &str = "\u{eb55}"; // x
}

/// Tabler Icons (filled where available, falling back to outline).
#[allow(dead_code)]
pub mod fill {
    pub const CALENDAR: &str = "\u{fb27}"; // calendar-filled
    pub const CLOCK: &str = "\u{f73a}"; // clock-filled
    pub const CLOUD: &str = "\u{f673}"; // cloud-filled
    pub const FOLDER_SIMPLE: &str = "\u{f749}"; // folder-filled
    pub const GEAR: &str = "\u{f69e}"; // settings-filled
    pub const GLOBE: &str = "\u{101c7}"; // world-filled
    pub const HARD_DRIVES: &str = "\u{eb1f}"; // server (outline)
    pub const KEY: &str = "\u{fe28}"; // key-filled
    pub const LINUX_LOGO: &str = "\u{ef59}"; // brand-ubuntu (outline)
    pub const PENCIL_SIMPLE: &str = "\u{101cf}"; // pencil-filled
    pub const PLUG: &str = "\u{ebd9}"; // plug (outline)
    pub const PLUS: &str = "\u{101ce}"; // plus-filled
    pub const SHARE_FAT: &str = "\u{eb21}"; // share (outline)
    pub const SQUARES_FOUR: &str = "\u{fe1c}"; // layout-grid-filled
    pub const TAG: &str = "\u{ff02}"; // tag-filled
    pub const TERMINAL_WINDOW: &str = "\u{ebef}"; // terminal-2 (outline)
    pub const VAULT: &str = "\u{f763}"; // shield-lock-filled
}

pub use regular as bold;

/// Termius "Serial" icon: a DB-style comb connector (ꟺ).
#[allow(dead_code)]
pub fn paint_serial(painter: &Painter, c: Pos2, color: Color32) {
    let s = Stroke::new(1.9_f32, color);
    let (l, r, top, bot) = (c.x - 6.5, c.x + 6.5, c.y - 5.0, c.y + 5.0);
    painter.line_segment([pos2(l, top), pos2(l + 1.5, bot)], s);
    painter.line_segment([pos2(r, top), pos2(r - 1.5, bot)], s);
    painter.line_segment([pos2(l + 1.5, bot), pos2(r - 1.5, bot)], s);
    painter.line_segment([pos2(c.x - 2.2, top + 1.0), pos2(c.x - 2.2, bot)], s);
    painter.line_segment([pos2(c.x + 2.2, top + 1.0), pos2(c.x + 2.2, bot)], s);
}
