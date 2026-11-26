//! Termius-style Settings modal (General + Terminal + Sync Vault).

use crate::i18n::{Language, Strings, tr};
use crate::storage::gist_sync::SyncConfig;
use crate::storage::store::AppSettings;
use crate::ui::icons::{self, IconStyle, bold, fill, regular};
use crate::ui::popover_menu::{MenuEntry, MenuIcon, render_popover_menu};
use crate::ui::theme;
use egui::{Align2, Color32, CornerRadius, Margin, Rect, RichText, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    General,
    Terminal,
    SyncVault,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncOnboardingStep {
    ChooseMode,
    Configure { is_existing: bool },
}

pub struct SettingsModalState {
    pub open: bool,
    pub tab: SettingsTab,
    pub lang_menu_open: bool,
    pub sync_step: SyncOnboardingStep,
    pub token_input: String,
    pub show_token: bool,
    pub password_input: String,
    pub confirm_password_input: String,
    pub show_password: bool,
    pub sync_busy: bool,
    pub sync_status: Option<String>,
    pub sync_error: Option<String>,
}

impl Default for SettingsModalState {
    fn default() -> Self {
        Self {
            open: false,
            tab: SettingsTab::General,
            lang_menu_open: false,
            sync_step: SyncOnboardingStep::ChooseMode,
            token_input: String::new(),
            show_token: false,
            password_input: String::new(),
            confirm_password_input: String::new(),
            show_password: false,
            sync_busy: false,
            sync_status: None,
            sync_error: None,
        }
    }
}

pub enum SyncUiAction {
    None,
    CreateNew { token: String, password: String },
    ConnectExisting { token: String, password: String },
    SyncNow,
    ToggleAutoSync(bool),
    ToggleAutoUnlock(bool),
    Disconnect,
}

fn toggle_switch(ui: &mut Ui, id: &str, value: &mut bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(38.0, 22.0), Sense::click());
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    let mut changed = false;
    if resp.clicked() {
        *value = !*value;
        changed = true;
    }
    let anim = ui.ctx().animate_bool_with_time(ui.id().with(id), *value, 0.12);
    let bg = theme::shade(36).lerp_to_gamma(theme::ACCENT_BLUE, anim);
    ui.painter().rect_filled(rect, CornerRadius::same(11), bg);
    let knob_x = rect.left() + 11.0 + anim * 16.0;
    ui.painter()
        .circle_filled(pos2(knob_x, rect.center().y), 8.5, Color32::WHITE);
    changed
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
    settings: &mut AppSettings,
    sync_cfg: Option<&SyncConfig>,
) -> (SyncUiAction, bool) {
    let mut action = SyncUiAction::None;
    let mut settings_changed = false;
    if !state.open {
        return (action, false);
    }
    let s = tr(settings.language);

    let screen = ctx.screen_rect();
    egui::Area::new(egui::Id::new("settings_backdrop"))
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
                s.settings_title,
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
                (SettingsTab::General, s.settings_general, regular::GEAR, IconStyle::Regular),
                (SettingsTab::Terminal, s.settings_terminal, fill::TERMINAL_WINDOW, IconStyle::Fill),
                (SettingsTab::SyncVault, s.settings_sync_vault, regular::CLOUD, IconStyle::Regular),
            ];
            for (i, (t, label, glyph, style)) in tabs.iter().enumerate() {
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
                if *t == SettingsTab::SyncVault && sync_cfg.is_some() {
                    painter.circle_filled(pos2(item.right() - 14.0, item.center().y), 4.0, theme::ACCENT_GREEN);
                }
                if resp.clicked() {
                    state.tab = *t;
                }
            }

            painter.text(
                pos2(side_rect.left() + 22.0, side_rect.bottom() - 22.0),
                Align2::LEFT_CENTER,
                s.settings_version,
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
                SettingsTab::General => {
                    let mut lang_anchor: Option<Rect> = None;
                    egui::ScrollArea::vertical().show(&mut content_ui, |ui| {
                        settings_card(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.spacing_mut().item_spacing.y = 2.0;
                                    ui.label(
                                        RichText::new(s.general_language_title)
                                            .font(theme::semibold(13.5))
                                            .color(theme::TEXT_PRIMARY),
                                    );
                                    ui.label(
                                        RichText::new(s.general_language_desc)
                                            .font(theme::regular(12.0))
                                            .color(theme::TEXT_MUTED),
                                    );
                                });

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let (sel_rect, sel_resp) =
                                        ui.allocate_exact_size(vec2(126.0, 32.0), Sense::click());
                                    let sel_resp = sel_resp.on_hover_cursor(egui::CursorIcon::PointingHand);
                                    let bg = if state.lang_menu_open || sel_resp.hovered() {
                                        theme::shade(22)
                                    } else {
                                        theme::shade(14)
                                    };
                                    ui.painter().rect_filled(sel_rect, CornerRadius::same(7), bg);
                                    ui.painter().rect_stroke(
                                        sel_rect,
                                        CornerRadius::same(7),
                                        Stroke::new(1.0_f32, theme::shade(18)),
                                        StrokeKind::Inside,
                                    );
                                    ui.painter().text(
                                        pos2(sel_rect.left() + 12.0, sel_rect.center().y),
                                        Align2::LEFT_CENTER,
                                        settings.language.label(),
                                        theme::medium(13.0),
                                        theme::TEXT_PRIMARY,
                                    );
                                    icons::paint(
                                        ui.painter(),
                                        pos2(sel_rect.right() - 14.0, sel_rect.center().y + 0.5),
                                        bold::CARET_DOWN,
                                        11.0,
                                        theme::TEXT_MUTED,
                                        IconStyle::Bold,
                                    );
                                    if sel_resp.clicked() {
                                        state.lang_menu_open = !state.lang_menu_open;
                                    }
                                    lang_anchor = Some(sel_rect);
                                });
                            });
                        });
                    });

                    if let Some(anchor) = lang_anchor
                        && let Some(id) = render_popover_menu(
                            ctx,
                            "settings_lang_popover",
                            anchor,
                            148.0,
                            &mut state.lang_menu_open,
                            &[
                                MenuEntry::Item {
                                    id: 0,
                                    icon: MenuIcon::Badge("EN"),
                                    label: "English",
                                    selected: settings.language == Language::En,
                                },
                                MenuEntry::Item {
                                    id: 1,
                                    icon: MenuIcon::Badge("FR"),
                                    label: "Français",
                                    selected: settings.language == Language::Fr,
                                },
                            ],
                        ) {
                            let next = if id == 0 { Language::En } else { Language::Fr };
                            if settings.language != next {
                                settings.language = next;
                                settings_changed = true;
                            }
                        }
                }
                SettingsTab::Terminal => {
                    egui::ScrollArea::vertical().show(&mut content_ui, |ui| {
                        settings_card(ui, |ui| {
                            ui.label(
                                RichText::new(s.term_settings_title)
                                    .font(theme::semibold(14.0))
                                    .color(theme::TEXT_PRIMARY),
                            );
                            ui.add_space(14.0);

                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(s.term_copy_on_select)
                                        .font(theme::regular(13.0))
                                        .color(theme::TEXT_PRIMARY),
                                );
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if toggle_switch(ui, "set_copy_on_select", &mut settings.copy_on_select) {
                                        settings_changed = true;
                                    }
                                });
                            });
                            ui.add_space(10.0);

                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(s.term_bright_bold)
                                        .font(theme::regular(13.0))
                                        .color(theme::TEXT_PRIMARY),
                                );
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if toggle_switch(ui, "set_bright_bold", &mut settings.bright_bold) {
                                        settings_changed = true;
                                    }
                                });
                            });
                        });

                        ui.add_space(14.0);

                        settings_card(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(s.term_text_size)
                                        .font(theme::regular(13.0))
                                        .color(theme::TEXT_PRIMARY),
                                );
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let plus_btn = egui::Button::new("+").min_size(vec2(28.0, 28.0));
                                    if ui.add(plus_btn).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                        settings.text_size = (settings.text_size + 1.0).min(28.0);
                                        settings_changed = true;
                                    }
                                    ui.label(
                                        RichText::new(format!("{}", settings.text_size as i32))
                                            .font(theme::medium(13.0))
                                            .color(theme::TEXT_PRIMARY),
                                    );
                                    let minus_btn = egui::Button::new("-").min_size(vec2(28.0, 28.0));
                                    if ui.add(minus_btn).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                        settings.text_size = (settings.text_size - 1.0).max(9.0);
                                        settings_changed = true;
                                    }
                                });
                            });
                        });
                    });
                }
                SettingsTab::SyncVault => {
                    egui::ScrollArea::vertical().show(&mut content_ui, |ui| {
                        action = render_sync_vault_tab(
                            ui,
                            state,
                            sync_cfg,
                            settings.auto_unlock_password.is_some(),
                            s,
                        );
                    });
                }
            }
        });

    (action, settings_changed)
}

const GITHUB_NEW_TOKEN_URL: &str =
    "https://github.com/settings/tokens/new?scopes=gist&description=Rustmius+Sync+Vault";

fn styled_secret_field(
    ui: &mut Ui,
    id: &str,
    text: &mut String,
    hint: &str,
    leading_icon: &str,
    show: &mut bool,
    allow_toggle: bool,
) {
    let h = 36.0;
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(w, h), Sense::hover());

    ui.painter().rect_filled(
        rect,
        CornerRadius::same(8),
        Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, 200),
    );

    icons::paint(
        ui.painter(),
        pos2(rect.left() + 15.0, rect.center().y),
        leading_icon,
        14.5,
        theme::TEXT_MUTED,
        IconStyle::Regular,
    );

    let right_pad = if allow_toggle { 34.0 } else { 10.0 };
    let edit_rect = Rect::from_min_max(
        pos2(rect.left() + 32.0, rect.top()),
        pos2(rect.right() - right_pad, rect.bottom()),
    );
    let font = theme::regular(13.0);
    let row_h = ui.fonts(|f| f.row_height(&font));
    let v_pad = ((edit_rect.height() - row_h) * 0.5).max(0.0) as i8;
    let resp = ui.put(
        edit_rect,
        egui::TextEdit::singleline(text)
            .id_salt(id)
            .password(!*show)
            .frame(false)
            .margin(Margin::symmetric(0, v_pad))
            .font(font.clone())
            .text_color(theme::TEXT_PRIMARY)
            .desired_width(edit_rect.width())
            .hint_text(
                RichText::new(hint)
                    .color(theme::TEXT_PLACEHOLDER)
                    .font(font),
            ),
    );

    if allow_toggle {
        let eye_r = Rect::from_center_size(pos2(rect.right() - 18.0, rect.center().y), vec2(26.0, 26.0));
        let eye_resp = ui
            .interact(eye_r, ui.id().with((id, "eye")), Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        if eye_resp.hovered() {
            ui.painter().rect_filled(eye_r, CornerRadius::same(6), theme::shade(12));
        }
        let glyph = if *show { regular::EYE_SLASH } else { regular::EYE };
        icons::paint(ui.painter(), eye_r.center(), glyph, 14.5, theme::TEXT_MUTED, IconStyle::Regular);
        if eye_resp.clicked() {
            *show = !*show;
        }
    }

    let stroke = if resp.has_focus() {
        Stroke::new(1.5_f32, theme::FOCUS_RING)
    } else {
        Stroke::new(1.0_f32, theme::shade(24))
    };
    ui.painter()
        .rect_stroke(rect, CornerRadius::same(8), stroke, StrokeKind::Inside);
}

fn choice_card(
    ui: &mut Ui,
    accent: Color32,
    glyph: &str,
    title: &str,
    subtitle: &str,
) -> bool {
    let w = ui.available_width();
    let h = 68.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(w, h), Sense::click());
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    let painter = ui.painter();

    let shadow_a = if resp.hovered() { 26 } else { 14 };
    painter.rect_filled(
        rect.translate(vec2(0.0, 3.0)),
        CornerRadius::same(10),
        Color32::from_black_alpha(shadow_a),
    );
    let bg = if resp.hovered() {
        Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, 235)
    } else {
        Color32::from_rgba_unmultiplied(0xF8, 0xFA, 0xFB, 210)
    };
    painter.rect_filled(rect, CornerRadius::same(10), bg);
    let border = if resp.hovered() {
        Stroke::new(1.2_f32, theme::ACCENT_BLUE)
    } else {
        Stroke::new(1.0_f32, theme::shade(16))
    };
    painter.rect_stroke(rect, CornerRadius::same(10), border, StrokeKind::Inside);

    let badge = Rect::from_min_size(pos2(rect.left() + 14.0, rect.center().y - 20.0), vec2(40.0, 40.0));
    painter.rect_filled(badge, CornerRadius::same(9), accent);
    icons::paint(painter, badge.center(), glyph, 20.0, Color32::WHITE, IconStyle::Fill);

    let tx = badge.right() + 14.0;
    painter.text(
        pos2(tx, rect.center().y - 10.0),
        Align2::LEFT_CENTER,
        title,
        theme::semibold(14.0),
        theme::TEXT_PRIMARY,
    );
    painter.text(
        pos2(tx, rect.center().y + 10.0),
        Align2::LEFT_CENTER,
        subtitle,
        theme::regular(12.0),
        theme::TEXT_MUTED,
    );

    resp.clicked()
}

fn render_sync_vault_tab(
    ui: &mut Ui,
    state: &mut SettingsModalState,
    sync_cfg: Option<&SyncConfig>,
    auto_unlock: bool,
    s: &Strings,
) -> SyncUiAction {
    let mut action = SyncUiAction::None;

    if let Some(cfg) = sync_cfg {
        settings_card(ui, |ui| {
            ui.horizontal(|ui| {
                let (badge, _) = ui.allocate_exact_size(vec2(38.0, 38.0), Sense::hover());
                ui.painter().rect_filled(badge, CornerRadius::same(9), theme::ACCENT_GREEN);
                icons::paint(ui.painter(), badge.center(), fill::CLOUD, 20.0, Color32::WHITE, IconStyle::Fill);
                ui.add_space(6.0);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.label(
                        RichText::new(s.sync_active_title)
                            .font(theme::semibold(14.5))
                            .color(theme::TEXT_PRIMARY),
                    );
                    ui.label(
                        RichText::new(s.sync_active_desc)
                            .font(theme::regular(12.0))
                            .color(theme::TEXT_MUTED),
                    );
                });
            });

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("Gist ID : {}", cfg.gist_id))
                        .font(theme::regular(12.0))
                        .color(theme::TEXT_MUTED),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let open_btn = egui::Button::new(
                        RichText::new(s.sync_open_github)
                            .font(theme::medium(12.0))
                            .color(theme::ACCENT_BLUE),
                    )
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::NONE);
                    if ui
                        .add(open_btn)
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        ui.ctx().open_url(egui::OpenUrl::new_tab(format!(
                            "https://gist.github.com/{}",
                            cfg.gist_id
                        )));
                    }
                });
            });

            let last = cfg
                .last_sync
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                .unwrap_or_else(|| s.sync_never.to_string());
            ui.label(
                RichText::new(format!("{} : {last}", s.sync_last_sync))
                    .font(theme::regular(12.0))
                    .color(theme::TEXT_MUTED),
            );

            ui.add_space(14.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(s.sync_auto)
                        .font(theme::regular(13.0))
                        .color(theme::TEXT_PRIMARY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut auto = cfg.auto_sync;
                    if toggle_switch(ui, "set_auto_sync", &mut auto) {
                        action = SyncUiAction::ToggleAutoSync(auto);
                    }
                });
            });

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(s.sync_auto_unlock)
                        .font(theme::regular(13.0))
                        .color(theme::TEXT_PRIMARY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut unlock = auto_unlock;
                    if toggle_switch(ui, "set_auto_unlock", &mut unlock) {
                        action = SyncUiAction::ToggleAutoUnlock(unlock);
                    }
                });
            });

            if let Some(status) = &state.sync_status {
                ui.add_space(8.0);
                ui.label(RichText::new(status).color(theme::ACCENT_BLUE).size(12.5));
            }
            if let Some(err) = &state.sync_error {
                ui.add_space(8.0);
                ui.label(RichText::new(err).color(theme::DANGER).size(12.5));
            }

            ui.add_space(16.0);
            ui.horizontal(|ui| {
                let sync_btn = egui::Button::new(
                    RichText::new(if state.sync_busy { s.sync_in_progress } else { s.sync_now })
                        .color(Color32::WHITE)
                        .font(theme::semibold(13.0)),
                )
                .fill(theme::ACCENT_BLUE)
                .corner_radius(CornerRadius::same(7))
                .min_size(vec2(124.0, 34.0));
                if ui
                    .add_enabled(!state.sync_busy, sync_btn)
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    action = SyncUiAction::SyncNow;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let disc_btn = egui::Button::new(
                        RichText::new(s.sync_disconnect).color(theme::DANGER).font(theme::medium(13.0)),
                    )
                    .fill(theme::shade(16))
                    .corner_radius(CornerRadius::same(7))
                    .min_size(vec2(110.0, 34.0));
                    if ui
                        .add_enabled(!state.sync_busy, disc_btn)
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        action = SyncUiAction::Disconnect;
                    }
                });
            });
        });
        return action;
    }

    match state.sync_step {
        SyncOnboardingStep::ChooseMode => {
            ui.horizontal(|ui| {
                let (badge, _) = ui.allocate_exact_size(vec2(44.0, 44.0), Sense::hover());
                ui.painter().rect_filled(badge, CornerRadius::same(11), theme::ACCENT_BLUE);
                icons::paint(ui.painter(), badge.center(), fill::CLOUD, 22.0, Color32::WHITE, IconStyle::Fill);
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.label(
                        RichText::new(s.sync_hero_title)
                            .font(theme::semibold(16.0))
                            .color(theme::TEXT_PRIMARY),
                    );
                    ui.label(
                        RichText::new(s.sync_hero_desc)
                            .font(theme::regular(12.5))
                            .color(theme::TEXT_MUTED),
                    );
                });
            });

            ui.add_space(18.0);
            ui.label(
                RichText::new(s.sync_question)
                    .font(theme::semibold(13.5))
                    .color(theme::TEXT_PRIMARY),
            );
            ui.add_space(10.0);

            if choice_card(
                ui,
                theme::ACCENT_BLUE,
                fill::PLUS,
                s.sync_new_title,
                s.sync_new_desc,
            ) {
                state.sync_error = None;
                state.sync_status = None;
                state.sync_step = SyncOnboardingStep::Configure { is_existing: false };
            }

            ui.add_space(10.0);

            if choice_card(
                ui,
                theme::ACCENT_GREEN,
                fill::VAULT,
                s.sync_existing_title,
                s.sync_existing_desc,
            ) {
                state.sync_error = None;
                state.sync_status = None;
                state.sync_step = SyncOnboardingStep::Configure { is_existing: true };
            }
        }
        SyncOnboardingStep::Configure { is_existing } => {
            settings_card(ui, |ui| {
                ui.horizontal(|ui| {
                    let (badge, _) = ui.allocate_exact_size(vec2(34.0, 34.0), Sense::hover());
                    ui.painter().rect_filled(
                        badge,
                        CornerRadius::same(8),
                        if is_existing { theme::ACCENT_GREEN } else { theme::ACCENT_BLUE },
                    );
                    icons::paint(
                        ui.painter(),
                        badge.center(),
                        if is_existing { fill::VAULT } else { fill::CLOUD },
                        17.0,
                        Color32::WHITE,
                        IconStyle::Fill,
                    );
                    ui.add_space(6.0);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 1.0;
                        let heading = if is_existing {
                            s.sync_existing_title
                        } else {
                            s.sync_new_title
                        };
                        ui.label(
                            RichText::new(heading)
                                .font(theme::semibold(14.5))
                                .color(theme::TEXT_PRIMARY),
                        );
                        ui.label(
                            RichText::new(s.sync_step_2)
                                .font(theme::regular(11.5))
                                .color(theme::TEXT_MUTED),
                        );
                    });
                });

                ui.add_space(16.0);

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(s.sync_token_label)
                            .font(theme::medium(12.5))
                            .color(theme::TEXT_PRIMARY),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let gen_btn = egui::Button::new(
                            RichText::new(s.sync_generate_token)
                                .font(theme::semibold(11.5))
                                .color(theme::ACCENT_BLUE),
                        )
                        .fill(theme::ACCENT_BLUE.linear_multiply(0.12))
                        .stroke(Stroke::new(1.0_f32, theme::ACCENT_BLUE.linear_multiply(0.3)))
                        .corner_radius(CornerRadius::same(6));
                        if ui
                            .add(gen_btn)
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            ui.ctx().open_url(egui::OpenUrl::new_tab(GITHUB_NEW_TOKEN_URL));
                        }
                    });
                });
                ui.add_space(6.0);
                styled_secret_field(
                    ui,
                    "sync_token_input",
                    &mut state.token_input,
                    "ghp_xxxxxxxxxxxxxxxxxxxx",
                    regular::KEY,
                    &mut state.show_token,
                    true,
                );

                ui.add_space(12.0);
                ui.label(
                    RichText::new(s.sync_master_pass)
                        .font(theme::medium(12.5))
                        .color(theme::TEXT_PRIMARY),
                );
                ui.add_space(6.0);
                styled_secret_field(
                    ui,
                    "sync_pass_input",
                    &mut state.password_input,
                    s.panel_password,
                    regular::LOCK,
                    &mut state.show_password,
                    true,
                );

                if !is_existing {
                    ui.add_space(10.0);
                    styled_secret_field(
                        ui,
                        "sync_pass_confirm",
                        &mut state.confirm_password_input,
                        s.sync_master_pass_confirm,
                        regular::LOCK,
                        &mut state.show_password,
                        false,
                    );
                }

                if let Some(status) = &state.sync_status {
                    ui.add_space(10.0);
                    ui.label(RichText::new(status).color(theme::ACCENT_BLUE).size(12.5));
                }
                if let Some(err) = &state.sync_error {
                    ui.add_space(10.0);
                    ui.label(RichText::new(err).color(theme::DANGER).size(12.5));
                }

                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    let back_btn = egui::Button::new(
                        RichText::new(s.sync_back)
                            .font(theme::medium(13.0))
                            .color(theme::TEXT_PRIMARY),
                    )
                    .fill(theme::shade(16))
                    .corner_radius(CornerRadius::same(7))
                    .min_size(vec2(92.0, 36.0));
                    if ui
                        .add_enabled(!state.sync_busy, back_btn)
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        state.sync_error = None;
                        state.sync_step = SyncOnboardingStep::ChooseMode;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let cta_text = if state.sync_busy {
                            s.sync_connecting
                        } else if is_existing {
                            s.sync_find_connect
                        } else {
                            s.sync_create_sync
                        };
                        let cta = egui::Button::new(
                            RichText::new(cta_text)
                                .color(Color32::WHITE)
                                .font(theme::semibold(13.0)),
                        )
                        .fill(theme::ACCENT_BLUE)
                        .corner_radius(CornerRadius::same(7))
                        .min_size(vec2(180.0, 36.0));

                        if ui
                            .add_enabled(!state.sync_busy, cta)
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            if state.token_input.trim().is_empty() {
                                state.sync_error = Some(s.sync_err_token.to_string());
                            } else if state.password_input.is_empty() {
                                state.sync_error = Some(s.sync_err_pass.to_string());
                            } else if !is_existing && state.password_input != state.confirm_password_input {
                                state.sync_error = Some(s.sync_err_pass_mismatch.to_string());
                            } else {
                                state.sync_error = None;
                                if is_existing {
                                    action = SyncUiAction::ConnectExisting {
                                        token: state.token_input.trim().to_string(),
                                        password: state.password_input.clone(),
                                    };
                                } else {
                                    action = SyncUiAction::CreateNew {
                                        token: state.token_input.trim().to_string(),
                                        password: state.password_input.clone(),
                                    };
                                }
                            }
                        }
                    });
                });
            });
        }
    }

    action
}
