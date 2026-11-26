//! OS & Linux distribution badge colors and vector icons (Termius style).

use crate::storage::{HostSource, OsKind};
use crate::ui::icons::{self, IconStyle, fill};
use crate::ui::theme;
use egui::{Align2, Color32, CornerRadius, Painter, Pos2, Rect, Stroke, pos2, vec2};

pub fn os_badge_color(os: OsKind, source: &HostSource) -> Color32 {
    match os {
        OsKind::Debian => Color32::from_rgb(0xB8, 0x00, 0x4C),
        OsKind::Ubuntu => Color32::from_rgb(0xE9, 0x54, 0x20),
        OsKind::Arch => Color32::from_rgb(0x17, 0x93, 0xD1),
        OsKind::Fedora => Color32::from_rgb(0x3C, 0x6E, 0xB4),
        OsKind::NixOS => Color32::from_rgb(0x52, 0x77, 0xC3),
        OsKind::Alpine => Color32::from_rgb(0x0D, 0x59, 0x7F),
        OsKind::CentOs => Color32::from_rgb(0x93, 0x22, 0x79),
        OsKind::RedHat => Color32::from_rgb(0xEE, 0x00, 0x00),
        OsKind::Rocky => Color32::from_rgb(0x10, 0xB9, 0x81),
        OsKind::Alma => Color32::from_rgb(0x0F, 0x42, 0x66),
        OsKind::Kali => Color32::from_rgb(0x26, 0x8B, 0xEE),
        OsKind::Mint => Color32::from_rgb(0x69, 0xB5, 0x3F),
        OsKind::OpenSuse => Color32::from_rgb(0x73, 0xBA, 0x25),
        OsKind::Gentoo => Color32::from_rgb(0x54, 0x48, 0x7A),
        OsKind::Manjaro => Color32::from_rgb(0x35, 0xBF, 0x5C),
        OsKind::FreeBsd => Color32::from_rgb(0xAB, 0x2B, 0x28),
        OsKind::MacOs => Color32::from_rgb(0x2E, 0x34, 0x40),
        OsKind::Windows => Color32::from_rgb(0x00, 0x78, 0xD4),
        OsKind::Linux => Color32::from_rgb(0x24, 0x34, 0x47),
        OsKind::Unknown => match source {
            HostSource::SshConfig => theme::ACCENT_BLUE,
            HostSource::Manual => theme::ACCENT_ORANGE,
        },
    }
}

pub fn paint_os_tile(painter: &Painter, rect: Rect, os: OsKind, source: &HostSource) {
    let bg = os_badge_color(os, source);
    painter.rect_filled(rect, CornerRadius::same(8), bg);
    paint_os_symbol(painter, rect.center(), os, 1.0);
}

pub fn paint_os_mini_badge(painter: &Painter, center: Pos2, os: OsKind) {
    let rect = Rect::from_center_size(center, vec2(18.0, 18.0));
    let bg = os_badge_color(os, &HostSource::Manual);
    painter.rect_filled(rect, CornerRadius::same(4), bg);
    paint_os_symbol(painter, center, os, 0.52);
}

fn paint_os_symbol(painter: &Painter, c: Pos2, os: OsKind, scale: f32) {
    let fg = Color32::WHITE;
    match os {
        OsKind::Debian => paint_debian_swirl(painter, c, fg, scale),
        OsKind::Ubuntu => paint_ubuntu(painter, c, fg, scale),
        OsKind::Arch => paint_arch(painter, c, fg, scale),
        OsKind::NixOS => paint_nixos(painter, c, fg, scale),
        OsKind::Alpine => paint_alpine(painter, c, fg, scale),
        OsKind::Windows => paint_windows(painter, c, fg, scale),
        OsKind::Fedora => {
            painter.text(
                c,
                Align2::CENTER_CENTER,
                "f",
                theme::semibold(22.0 * scale),
                fg,
            );
        }
        OsKind::RedHat => {
            painter.text(
                c,
                Align2::CENTER_CENTER,
                "RH",
                theme::semibold(14.0 * scale),
                fg,
            );
        }
        OsKind::CentOs => {
            painter.text(
                c,
                Align2::CENTER_CENTER,
                "C",
                theme::semibold(19.0 * scale),
                fg,
            );
        }
        OsKind::Rocky => {
            painter.text(
                c,
                Align2::CENTER_CENTER,
                "R",
                theme::semibold(19.0 * scale),
                fg,
            );
        }
        OsKind::Alma => {
            painter.text(
                c,
                Align2::CENTER_CENTER,
                "A",
                theme::semibold(19.0 * scale),
                fg,
            );
        }
        OsKind::Kali => {
            painter.text(
                c,
                Align2::CENTER_CENTER,
                "K",
                theme::semibold(19.0 * scale),
                fg,
            );
        }
        OsKind::Mint => {
            painter.text(
                c,
                Align2::CENTER_CENTER,
                "lm",
                theme::semibold(15.0 * scale),
                fg,
            );
        }
        OsKind::OpenSuse => {
            painter.text(
                c,
                Align2::CENTER_CENTER,
                "S",
                theme::semibold(19.0 * scale),
                fg,
            );
        }
        OsKind::Gentoo => {
            painter.text(
                c,
                Align2::CENTER_CENTER,
                "g",
                theme::semibold(20.0 * scale),
                fg,
            );
        }
        OsKind::Manjaro => {
            painter.text(
                c,
                Align2::CENTER_CENTER,
                "M",
                theme::semibold(18.0 * scale),
                fg,
            );
        }
        OsKind::FreeBsd => {
            painter.text(
                c,
                Align2::CENTER_CENTER,
                "BSD",
                theme::semibold(11.5 * scale),
                fg,
            );
        }
        OsKind::MacOs => {
            painter.text(
                c,
                Align2::CENTER_CENTER,
                "mac",
                theme::semibold(12.0 * scale),
                fg,
            );
        }
        OsKind::Linux => {
            icons::paint(
                painter,
                c,
                fill::LINUX_LOGO,
                22.0 * scale,
                fg,
                IconStyle::Fill,
            );
        }
        OsKind::Unknown => {
            icons::paint(
                painter,
                c,
                fill::HARD_DRIVES,
                20.0 * scale,
                fg,
                IconStyle::Fill,
            );
        }
    }
}

fn paint_debian_swirl(painter: &Painter, c: Pos2, fg: Color32, scale: f32) {
    let mut pts = Vec::with_capacity(48);
    let steps = 44;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let angle = -0.4 + t * std::f32::consts::TAU * 1.45;
        let r = (1.5 + t * 9.5) * scale;
        pts.push(pos2(
            c.x + r * angle.cos(),
            c.y + r * angle.sin() - 0.5 * scale,
        ));
    }
    painter.add(egui::Shape::line(
        pts,
        Stroke::new((2.0 * scale).max(1.2), fg),
    ));
}

fn paint_ubuntu(painter: &Painter, c: Pos2, fg: Color32, scale: f32) {
    painter.circle_stroke(c, 7.5 * scale, Stroke::new((2.2 * scale).max(1.2), fg));
    for k in 0..3 {
        let a = (k as f32) * std::f32::consts::TAU / 3.0 - std::f32::consts::FRAC_PI_2;
        let p = c + vec2(a.cos(), a.sin()) * (9.0 * scale);
        painter.circle_filled(p, (2.5 * scale).max(1.3), fg);
    }
}

fn paint_arch(painter: &Painter, c: Pos2, fg: Color32, scale: f32) {
    let s = Stroke::new((2.0 * scale).max(1.2), fg);
    let top = pos2(c.x, c.y - 10.0 * scale);
    let bl = pos2(c.x - 9.0 * scale, c.y + 9.0 * scale);
    let br = pos2(c.x + 9.0 * scale, c.y + 9.0 * scale);
    painter.line_segment([top, bl], s);
    painter.line_segment([top, br], s);
    painter.line_segment(
        [
            pos2(c.x - 5.0 * scale, c.y + 3.0 * scale),
            pos2(c.x + 5.0 * scale, c.y + 3.0 * scale),
        ],
        s,
    );
}

fn paint_nixos(painter: &Painter, c: Pos2, fg: Color32, scale: f32) {
    let s = Stroke::new((1.8 * scale).max(1.1), fg);
    for k in 0..6 {
        let a = (k as f32) * std::f32::consts::FRAC_PI_3;
        let dir = vec2(a.cos(), a.sin());
        painter.line_segment([c + dir * (2.5 * scale), c + dir * (10.0 * scale)], s);
    }
}

fn paint_alpine(painter: &Painter, c: Pos2, fg: Color32, scale: f32) {
    let s = Stroke::new((2.0 * scale).max(1.2), fg);
    painter.line_segment(
        [
            pos2(c.x - 10.0 * scale, c.y + 6.0 * scale),
            pos2(c.x - 2.0 * scale, c.y - 6.0 * scale),
        ],
        s,
    );
    painter.line_segment(
        [
            pos2(c.x - 2.0 * scale, c.y - 6.0 * scale),
            pos2(c.x + 4.0 * scale, c.y + 6.0 * scale),
        ],
        s,
    );
    painter.line_segment(
        [
            pos2(c.x + 1.0 * scale, c.y + 1.0 * scale),
            pos2(c.x + 5.0 * scale, c.y - 4.0 * scale),
        ],
        s,
    );
    painter.line_segment(
        [
            pos2(c.x + 5.0 * scale, c.y - 4.0 * scale),
            pos2(c.x + 10.0 * scale, c.y + 6.0 * scale),
        ],
        s,
    );
}

fn paint_windows(painter: &Painter, c: Pos2, fg: Color32, scale: f32) {
    let gap = 1.5 * scale;
    let sz = 7.0 * scale;
    for (dx, dy) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        let x0 = if dx < 0.0 { c.x - gap - sz } else { c.x + gap };
        let y0 = if dy < 0.0 { c.y - gap - sz } else { c.y + gap };
        painter.rect_filled(Rect::from_min_size(pos2(x0, y0), vec2(sz, sz)), 1.0, fg);
    }
}
