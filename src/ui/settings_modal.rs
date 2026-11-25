//! Termius-style Settings modal (Terminal + Sync Vault).

use crate::ui::icons::{self, IconStyle, fill, regular};
use crate::ui::theme;
use egui::{Align2, Color32, CornerRadius, Margin, Rect, RichText, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    Terminal,
    SyncVault,
}

#[derive(Debug, Clone)]
pub struct TerminalSettings {
    pub text_size: f32,
    pub copy_on_select: bool,
    pub bright_bold: bool,
}

impl Default for TerminalSettings {
    fn default() -> Self {
        Self {
            text_size: 14.0,
            copy_on_select: false,
            bright_bold: false,
        }
    }
}

pub struct SettingsModalState {
    pub open: bool,
    pub tab: SettingsTab,
}

impl Default for SettingsModalState {
    fn default() -> Self {
        Self {
            open: false,
            tab: SettingsTab::Terminal,
        }
    }
}

fn toggle_switch(ui: &mut Ui, id: &str, value: &mut bool) {
    let (rect, resp) = ui.allocate_exact_size(vec2(38.0, 22.0), Sense::click());
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    if resp.clicked() {
        *value = !*value;
    }
    let anim = ui.ctx().animate_bool_with_time(ui.id().with(id), *value, 0.12);
    let bg = theme::shade(36).lerp_to_gamma(theme::ACCENT_BLUE, anim);
    ui.painter().rect_filled(rect, CornerRadius::same(11), bg);
    let knob_x = rect.left() + 11.0 + anim * 16.0;
    ui.painter()
        .circle_filled(pos2(knob_x, rect.center().y), 8.5, Color32::WHITE);
}

fn settings_card(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    egui::Frame::NONE
        .fill(theme::light(110))
        .stroke(Stroke::new(1.0_f32, theme::shade(14)))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add_contents(ui);
        });
}

pub fn render_settings_modal(
    ctx: &egui::Context,
    state: &mut SettingsModalState,
    settings: &mut TerminalSettings,
) {
    if !state.open {
        return;
    }

    let screen = ctx.screen_rect();
    let backdrop = egui::Area::new(egui::Id::new("settings_backdrop"))
        .order(egui::Order::Middle)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let resp = ui.allocate_response(screen.size(), Sense::click());
            ui.painter()
                .rect_filled(screen, 0.0, Color32::from_black_alpha(90));
            if resp.clicked() {
                state.open = false;
            }
        });
    let _ = backdrop;

    let modal_w = (screen.width() - 80.0).min(820.0);
    let modal_h = (screen.height() - 80.0).min(540.0);
    let modal_rect = Rect::from_center_size(screen.center(), vec2(modal_w, modal_h));

    egui::Area::new(egui::Id::new("settings_modal"))
        .order(egui::Order::Foreground)
        .fixed_pos(modal_rect.min)
        .show(ctx, |ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(modal_w, modal_h), Sense::hover());
            let painter = ui.painter();
            painter.rect_filled(
                rect.translate(vec2(0.0, 6.0)),
                CornerRadius::same(12),
                Color32::from_black_alpha(45),
            );
            painter.rect_filled(rect, CornerRadius::same(12), Color32::from_rgb(0xE8, 0xED, 0xEF));
            painter.rect_stroke(
                rect,
                CornerRadius::same(12),
                Stroke::new(1.0_f32, theme::shade(24)),
                StrokeKind::Inside,
            );

            // Top bar of modal
            let header_h = 46.0;
            let header = Rect::from_min_size(rect.min, vec2(modal_w, header_h));
            painter.hline(
                header.x_range(),
                header.bottom(),
                Stroke::new(1.0_f32, theme::SEPARATOR),
            );
            painter.text(
                pos2(header.left() + 20.0, header.center().y),
                Align2::LEFT_CENTER,
                "Settings",
                theme::semibold(14.5),
                theme::TEXT_PRIMARY,
            );

            let close_r = Rect::from_center_size(pos2(header.right() - 24.0, header.center().y), vec2(28.0, 28.0));
            let close_resp = ui
                .interact(close_r, ui.id().with("settings_close"), Sense::click())
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            if close_resp.hovered() {
                painter.rect_filled(close_r, CornerRadius::same(6), theme::shade(16));
            }
            icons::paint(
                painter,
                close_r.center(),
                regular::X,
                14.0,
                theme::TEXT_MUTED,
                IconStyle::Regular,
            );
            if close_resp.clicked() {
                state.open = false;
            }

            // Left sidebar inside modal
            let side_w = 186.0;
            let side_rect = Rect::from_min_max(pos2(rect.left(), header.bottom()), pos2(rect.left() + side_w, rect.bottom()));
            painter.rect_filled(side_rect, CornerRadius { nw: 0, ne: 0, sw: 12, se: 0 }, theme::light(40));
            painter.vline(side_rect.right(), side_rect.y_range(), Stroke::new(1.0_f32, theme::SEPARATOR));

            let tabs = [
                (SettingsTab::Terminal, "Terminal", fill::TERMINAL_WINDOW, IconStyle::Fill, None),
                (SettingsTab::SyncVault, "Sync Vault", regular::CLOUD, IconStyle::Regular, Some("Soon")),
            ];
            for (i, (t, label, glyph, style, badge)) in tabs.iter().enumerate() {
                let item = Rect::from_min_size(
                    pos2(side_rect.left() + 10.0, side_rect.top() + 14.0 + i as f32 * 40.0),
                    vec2(side_w - 20.0, 34.0),
                );
                let resp = ui
                    .interact(item, ui.id().with(("set_tab", i)), Sense::click())
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if state.tab == *t {
                    painter.rect_filled(item, CornerRadius::same(7), theme::shade(18));
                } else if resp.hovered() {
                    painter.rect_filled(item, CornerRadius::same(7), theme::shade(10));
                }
                icons::paint(
                    painter,
                    pos2(item.left() + 18.0, item.center().y),
                    glyph,
                    15.0,
                    theme::ICON_DARK,
                    *style,
                );
                painter.text(
                    pos2(item.left() + 36.0, item.center().y),
                    Align2::LEFT_CENTER,
                    *label,
                    theme::medium(13.5),
                    theme::TEXT_PRIMARY,
                );
                if let Some(b) = badge {
                    let br = Rect::from_center_size(pos2(item.right() - 24.0, item.center().y), vec2(36.0, 18.0));
                    painter.rect_filled(br, CornerRadius::same(4), theme::shade(18));
                    painter.text(br.center(), Align2::CENTER_CENTER, *b, theme::medium(10.5), theme::TEXT_MUTED);
                }
                if resp.clicked() {
                    state.tab = *t;
                }
            }

            painter.text(
                pos2(side_rect.left() + 22.0, side_rect.bottom() - 22.0),
                Align2::LEFT_CENTER,
                "Version",
                theme::regular(12.0),
                theme::TEXT_MUTED,
            );
            painter.text(
                pos2(side_rect.right() - 18.0, side_rect.bottom() - 22.0),
                Align2::RIGHT_CENTER,
                env!("CARGO_PKG_VERSION"),
                theme::regular(12.0),
                theme::TEXT_MUTED,
            );

            // Right content
            let content_rect = Rect::from_min_max(pos2(side_rect.right(), header.bottom()), rect.max).shrink(24.0);
            let mut content_ui = ui.new_child(egui::UiBuilder::new().max_rect(content_rect));

            match state.tab {
                SettingsTab::Terminal => {
                    egui::ScrollArea::vertical().show(&mut content_ui, |ui| {
                        settings_card(ui, |ui| {
                            ui.label(
                                RichText::new("Terminal settings")
                                    .font(theme::semibold(14.0))
                                    .color(theme::TEXT_PRIMARY),
                            );
                            ui.add_space(14.0);

                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("Select text to copy & Right click to paste")
                                        .font(theme::regular(13.0))
                                        .color(theme::TEXT_PRIMARY),
                                );
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    toggle_switch(ui, "set_copy_on_select", &mut settings.copy_on_select);
                                });
                            });
                            ui.add_space(10.0);

                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("Use bright colours for bold text")
                                        .font(theme::regular(13.0))
                                        .color(theme::TEXT_PRIMARY),
                                );
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    toggle_switch(ui, "set_bright_bold", &mut settings.bright_bold);
                                });
                            });
                        });

                        ui.add_space(14.0);

                        settings_card(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("Text Size")
                                        .font(theme::regular(13.0))
                                        .color(theme::TEXT_PRIMARY),
                                );
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let plus_btn = egui::Button::new("+").min_size(vec2(28.0, 28.0));
                                    if ui.add(plus_btn).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                        settings.text_size = (settings.text_size + 1.0).min(28.0);
                                    }
                                    ui.label(
                                        RichText::new(format!("{}", settings.text_size as i32))
                                            .font(theme::medium(13.0))
                                            .color(theme::TEXT_PRIMARY),
                                    );
                                    let minus_btn = egui::Button::new("-").min_size(vec2(28.0, 28.0));
                                    if ui.add(minus_btn).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                        settings.text_size = (settings.text_size - 1.0).max(9.0);
                                    }
                                });
                            });
                        });
                    });
                }
                SettingsTab::SyncVault => {
                    content_ui.centered_and_justified(|ui| {
                        ui.label(
                            RichText::new("Sync Vault (GitHub Gist chiffré) — Bientôt disponible")
                                .font(theme::medium(14.0))
                                .color(theme::TEXT_MUTED),
                        );
                    });
                }
            }
        });
}
