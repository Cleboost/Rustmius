//! Icon rendering (Phosphor glyphs + a few hand-drawn Termius-specific icons).

use crate::ui::theme;
use egui::{Align2, Color32, FontFamily, FontId, Painter, Pos2, Stroke, pos2};

pub use egui_phosphor::{bold, fill, regular};

#[derive(Clone, Copy)]
pub enum IconStyle {
    Regular,
    Fill,
    Bold,
}

pub fn font(size: f32, style: IconStyle) -> FontId {
    let family = match style {
        IconStyle::Regular => theme::PH_REGULAR,
        IconStyle::Fill => theme::PH_FILL,
        IconStyle::Bold => theme::PH_BOLD,
    };
    FontId::new(size, FontFamily::Name(family.into()))
}

/// Paints a Phosphor glyph centered on `center`.
pub fn paint(painter: &Painter, center: Pos2, glyph: &str, size: f32, color: Color32, style: IconStyle) {
    painter.text(center, Align2::CENTER_CENTER, glyph, font(size, style), color);
}

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
