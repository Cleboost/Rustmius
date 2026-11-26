//! Termius-style SSH connecting screen with animated spinner arc.

use crate::i18n::{Language, tr};
use crate::storage::{HostSource, OsKind};
use crate::ui::icons::{self, IconStyle, fill};
use crate::ui::os_icons::paint_os_tile;
use crate::ui::theme;
use egui::{Align2, Color32, CornerRadius, Rect, Sense, Stroke, Ui, pos2, vec2};

pub enum ConnectingAction {
    None,
    Close,
    ToggleLogs,
}

#[allow(clippy::too_many_arguments)]
pub fn render_connecting_screen(
    ui: &mut Ui,
    label: &str,
    hostname: &str,
    port: u16,
    os: OsKind,
    source: &HostSource,
    show_logs: bool,
    logs: &[String],
    lang: Language,
) -> ConnectingAction {
    let s = tr(lang);
    let mut action = ConnectingAction::None;
    let area = ui.max_rect();
    let painter = ui.painter_at(area);

    // Block centered horizontally, positioned in the upper-middle like Termius
    let block_w = 400.0;
    let left = (area.center().x - block_w * 0.5).max(area.left() + 24.0);
    let top = (area.center().y - 120.0).max(area.top() + 32.0);

    // 1. Header row: OS tile + Host label/SSH addr + "Show logs" button
    let tile = Rect::from_min_size(pos2(left, top), vec2(42.0, 42.0));
    paint_os_tile(&painter, tile, os, source);

    let tx = tile.right() + 14.0;
    painter.text(
        pos2(tx, tile.top() + 11.0),
        Align2::LEFT_CENTER,
        label,
        theme::semibold(15.0),
        theme::TEXT_PRIMARY,
    );
    painter.text(
        pos2(tx, tile.top() + 31.0),
        Align2::LEFT_CENTER,
        format!("SSH {hostname}:{port}"),
        theme::regular(12.5),
        theme::TEXT_MUTED,
    );

    let logs_btn = Rect::from_min_size(pos2(left + block_w - 102.0, top + 3.0), vec2(102.0, 36.0));
    let lresp = ui
        .interact(logs_btn, ui.id().with("conn_show_logs"), Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    painter.rect_filled(
        logs_btn,
        CornerRadius::same(8),
        theme::shade(if lresp.hovered() { 30 } else { 20 }),
    );
    painter.text(
        logs_btn.center(),
        Align2::CENTER_CENTER,
        if show_logs { s.hide_logs } else { s.show_logs },
        theme::semibold(13.0),
        theme::TEXT_PRIMARY,
    );
    if lresp.clicked() {
        action = ConnectingAction::ToggleLogs;
    }

    // 2. Connection line + animated left plug spinner + right terminal badge
    let line_y = top + 90.0;
    let left_c = pos2(left + 18.0, line_y);
    let right_c = pos2(left + block_w - 14.0, line_y);
    let slate = Color32::from_rgb(0x43, 0x4B, 0x5B);

    painter.line_segment(
        [
            pos2(left_c.x + 14.0, line_y),
            pos2(right_c.x - 13.0, line_y),
        ],
        Stroke::new(3.0_f32, slate),
    );

    // Animated rotating arc around left blue circle
    let t = ui.input(|i| i.time) as f32;
    let start_angle = t * 4.2;
    let arc_span = std::f32::consts::TAU * 0.72;
    let r_arc = 19.0;
    let mut arc_pts = Vec::with_capacity(32);
    for i in 0..=28 {
        let a = start_angle + (i as f32 / 28.0) * arc_span;
        arc_pts.push(pos2(left_c.x + r_arc * a.cos(), left_c.y + r_arc * a.sin()));
    }
    painter.add(egui::Shape::line(
        arc_pts,
        Stroke::new(3.0_f32, theme::ACCENT_BLUE),
    ));
    ui.ctx().request_repaint();

    // Left blue circle + plug icon
    painter.circle_filled(left_c, 14.0, theme::ACCENT_BLUE);
    icons::paint(
        &painter,
        left_c,
        fill::PLUG,
        15.0,
        Color32::WHITE,
        IconStyle::Fill,
    );

    // Right slate circle + ">_"
    painter.circle_filled(right_c, 13.0, slate);
    painter.text(
        right_c + vec2(0.0, -0.5),
        Align2::CENTER_CENTER,
        ">_",
        theme::semibold(11.0),
        Color32::WHITE,
    );

    // 3. Close button
    let close_btn = Rect::from_min_size(pos2(left, line_y + 38.0), vec2(82.0, 36.0));
    let cresp = ui
        .interact(close_btn, ui.id().with("conn_close"), Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    painter.rect_filled(
        close_btn,
        CornerRadius::same(8),
        theme::shade(if cresp.hovered() { 30 } else { 20 }),
    );
    painter.text(
        close_btn.center(),
        Align2::CENTER_CENTER,
        s.close,
        theme::semibold(13.0),
        theme::TEXT_PRIMARY,
    );
    if cresp.clicked() {
        action = ConnectingAction::Close;
    }

    // 4. Optional logs box
    if show_logs {
        let log_rect =
            Rect::from_min_size(pos2(left, close_btn.bottom() + 14.0), vec2(block_w, 140.0));
        painter.rect_filled(log_rect, CornerRadius::same(8), theme::TERM_BG);
        let mut log_ui = ui.new_child(egui::UiBuilder::new().max_rect(log_rect.shrink(10.0)));
        egui::ScrollArea::vertical().show(&mut log_ui, |ui| {
            for line in logs {
                ui.label(
                    egui::RichText::new(line)
                        .font(egui::FontId::monospace(11.5))
                        .color(theme::TERM_FG),
                );
            }
        });
    }

    action
}
