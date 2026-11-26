//! Termius design tokens.
//!
//! All metrics are logical pixels measured from a 1920×1028 Termius reference
//! capture (Linux, light "Vaults" view).

use egui::{
    Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, Style, Visuals,
};

// ---------------------------------------------------------------- layout ---
pub const TITLEBAR_H: f32 = 48.0;
pub const SIDEBAR_W: f32 = 184.0;
pub const HEADER_H: f32 = 106.0;

// ------------------------------------------------------------ title bar ---
pub const TOPBAR_LEFT: Color32 = Color32::from_rgb(0x24, 0x2A, 0x38);
pub const TOPBAR_MID: Color32 = Color32::from_rgb(0x2E, 0x37, 0x43);
pub const TOPBAR_RIGHT: Color32 = Color32::from_rgb(0x2A, 0x30, 0x3E);
pub const TOPBAR_TAB_ACTIVE: Color32 = Color32::from_rgb(0x3E, 0x43, 0x4F);
pub const TOPBAR_TAB_INACTIVE: Color32 = Color32::from_rgb(0x2F, 0x33, 0x3E);
pub const TOPBAR_TAB_HOVER: Color32 = Color32::from_rgb(0x37, 0x3C, 0x48);
pub const TOPBAR_TEXT: Color32 = Color32::from_rgb(0xE8, 0xEA, 0xF0);
pub const TOPBAR_MUTED: Color32 = Color32::from_rgb(0x96, 0x9A, 0xA5);
pub const TOPBAR_ICON: Color32 = Color32::from_rgb(0x9A, 0xA0, 0xAC);
pub const WINDOW_CLOSE_HOVER: Color32 = Color32::from_rgb(0xE8, 0x11, 0x23);

// ------------------------------------------------------------ workspace ---
pub const BG_BASE: Color32 = Color32::from_rgb(0xD5, 0xDA, 0xDB);
/// Translucent black overlays (premultiplied).
pub const fn shade(alpha: u8) -> Color32 {
    Color32::from_rgba_premultiplied(0, 0, 0, alpha)
}
/// Translucent white overlays (premultiplied).
pub const fn light(alpha: u8) -> Color32 {
    Color32::from_rgba_premultiplied(alpha, alpha, alpha, alpha)
}
pub const SEPARATOR: Color32 = shade(16);

pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0x1B, 0x21, 0x29);
pub const TEXT_SIDEBAR: Color32 = Color32::from_rgb(0x24, 0x29, 0x2E);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x66, 0x72, 0x7A);
pub const TEXT_PLACEHOLDER: Color32 = Color32::from_rgb(0x7D, 0x87, 0x8A);
pub const TEXT_DISABLED: Color32 = Color32::from_rgb(0x7C, 0x87, 0x89);
pub const ICON_DARK: Color32 = Color32::from_rgb(0x1D, 0x25, 0x2F);
#[allow(dead_code)]
pub const ICON_MUTED: Color32 = Color32::from_rgb(0x98, 0xA0, 0xA6);

pub const ACCENT_BLUE: Color32 = Color32::from_rgb(0x24, 0x96, 0xEE);
pub const ACCENT_BLUE_TOP: Color32 = Color32::from_rgb(0x2A, 0x9C, 0xF1);
pub const ACCENT_BLUE_BOTTOM: Color32 = Color32::from_rgb(0x1E, 0x8A, 0xE5);
pub const FOCUS_RING: Color32 = Color32::from_rgb(0x3D, 0x8F, 0xE0);
pub const ACCENT_GREEN: Color32 = Color32::from_rgb(0x00, 0xC4, 0x8C);
pub const ACCENT_ORANGE: Color32 = Color32::from_rgb(0xF2, 0x99, 0x4A);
pub const DANGER: Color32 = Color32::from_rgb(0xEF, 0x44, 0x44);

// ------------------------------------------------------------- terminal ---
pub const TERM_BG: Color32 = Color32::from_rgb(0x14, 0x17, 0x1F);
pub const TERM_FG: Color32 = Color32::from_rgb(0xE6, 0xEA, 0xF2);

// ----------------------------------------------------------------- fonts ---
const INTER_REGULAR: &str = "inter-regular";
const INTER_MEDIUM: &str = "inter-medium";
const INTER_SEMIBOLD: &str = "inter-semibold";
pub const TI_REGULAR: &str = "ti-regular";
pub const TI_FILLED: &str = "ti-filled";

pub fn regular(size: f32) -> FontId {
    FontId::proportional(size)
}
pub fn medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(INTER_MEDIUM.into()))
}
pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(INTER_SEMIBOLD.into()))
}

pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    let mut add = |name: &str, data: FontData| {
        fonts.font_data.insert(name.to_owned(), data.into());
    };
    add(
        INTER_REGULAR,
        FontData::from_static(include_bytes!("../../assets/fonts/Inter-Regular.ttf")),
    );
    add(
        INTER_MEDIUM,
        FontData::from_static(include_bytes!("../../assets/fonts/Inter-Medium.ttf")),
    );
    add(
        INTER_SEMIBOLD,
        FontData::from_static(include_bytes!("../../assets/fonts/Inter-SemiBold.ttf")),
    );
    add(
        TI_REGULAR,
        FontData::from_static(include_bytes!("../../assets/fonts/tabler-icons.ttf")),
    );
    add(
        TI_FILLED,
        FontData::from_static(include_bytes!("../../assets/fonts/tabler-icons-filled.ttf")),
    );

    let defaults = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();

    let family = |primary: &str| {
        let mut v = vec![primary.to_owned(), TI_REGULAR.to_owned()];
        v.extend(defaults.iter().cloned());
        v
    };

    fonts
        .families
        .insert(FontFamily::Proportional, family(INTER_REGULAR));
    fonts
        .families
        .insert(FontFamily::Name(INTER_MEDIUM.into()), family(INTER_MEDIUM));
    fonts.families.insert(
        FontFamily::Name(INTER_SEMIBOLD.into()),
        family(INTER_SEMIBOLD),
    );
    for ti in [TI_REGULAR, TI_FILLED] {
        fonts.families.insert(
            FontFamily::Name(ti.into()),
            vec![ti.to_owned(), TI_REGULAR.to_owned(), INTER_REGULAR.to_owned()],
        );
    }

    let mut mono = fonts
        .families
        .get(&FontFamily::Monospace)
        .cloned()
        .unwrap_or_default();
    let noto_emoji_vec = "noto-emoji-vector";
    add(
        noto_emoji_vec,
        FontData::from_static(include_bytes!("../../assets/fonts/NotoEmoji-Regular.ttf")),
    );
    mono.push(noto_emoji_vec.to_owned());
    for fallback in ["NotoEmoji-Regular", "emoji-icon-font"] {
        if fonts.font_data.contains_key(fallback) && !mono.iter().any(|f| f == fallback) {
            mono.push(fallback.to_owned());
        }
    }
    for (idx, path) in discover_system_mono_fallbacks().into_iter().enumerate() {
        if let Ok(bytes) = std::fs::read(&path) {
            let name = format!("system-mono-fallback-{idx}");
            fonts
                .font_data
                .insert(name.clone(), FontData::from_owned(bytes).into());
            mono.push(name);
        }
    }
    fonts.families.insert(FontFamily::Monospace, mono);

    ctx.set_fonts(fonts);
}

fn discover_system_mono_fallbacks() -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();
    let patterns = [
        "Noto Sans Mono CJK JP",
        "Noto Sans CJK JP",
        "DejaVu Sans Mono",
        "Noto Sans Symbols 2",
    ];
    for pattern in patterns {
        let Ok(output) = std::process::Command::new("fc-match")
            .args(["-f", "%{file}\n", pattern])
            .output()
        else {
            continue;
        };
        if !output.status.success() {
            continue;
        }
        let raw = String::from_utf8_lossy(&output.stdout);
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let path = std::path::PathBuf::from(trimmed);
        if path.is_file() && !paths.contains(&path) {
            paths.push(path);
        }
    }
    paths
}

pub fn apply_termius_theme(ctx: &egui::Context) {
    install_fonts(ctx);

    let mut style = Style::default();
    let mut visuals = Visuals::light();

    visuals.panel_fill = Color32::TRANSPARENT;
    visuals.window_fill = Color32::from_rgb(0xF4, 0xF6, 0xF7);
    visuals.window_stroke = Stroke::new(1.0_f32, shade(24));
    visuals.window_corner_radius = CornerRadius::same(10);
    visuals.faint_bg_color = light(60);
    visuals.extreme_bg_color = light(90);
    visuals.text_cursor.stroke = Stroke::new(1.5_f32, TEXT_PRIMARY);

    for w in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        w.corner_radius = CornerRadius::same(6);
        w.fg_stroke = Stroke::new(1.0_f32, TEXT_PRIMARY);
    }
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, shade(20));
    visuals.widgets.inactive.weak_bg_fill = shade(18);
    visuals.widgets.inactive.bg_fill = shade(18);
    visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    visuals.widgets.hovered.weak_bg_fill = shade(28);
    visuals.widgets.hovered.bg_fill = shade(28);
    visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    visuals.widgets.active.weak_bg_fill = shade(38);
    visuals.widgets.active.bg_fill = shade(38);
    visuals.widgets.active.bg_stroke = Stroke::NONE;

    visuals.selection.bg_fill = ACCENT_BLUE.linear_multiply(0.25);
    visuals.selection.stroke = Stroke::new(1.0_f32, FOCUS_RING);

    style.visuals = visuals;
    style.visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style
        .text_styles
        .insert(egui::TextStyle::Body, regular(13.0));
    style
        .text_styles
        .insert(egui::TextStyle::Button, medium(13.0));
    style
        .text_styles
        .insert(egui::TextStyle::Small, regular(11.0));
    style
        .text_styles
        .insert(egui::TextStyle::Heading, semibold(16.0));

    ctx.set_global_style(style);
}
