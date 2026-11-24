use crate::storage::{AuthMethod, Host, HostSource, OsKind};
use crate::ui::icons::{self, IconStyle, fill, regular};
use crate::ui::theme;
use chrono::Utc;
use egui::{Align, Color32, CornerRadius, Margin, RichText, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use uuid::Uuid;

pub struct HostModalState {
    pub editing_id: Option<Uuid>,
    pub label: String,
    pub hostname: String,
    pub port: String,
    pub username: String,
    pub auth_mode: usize, // 0 = Auto/Agent, 1 = Password, 2 = PrivateKey
    pub password: String,
    pub show_password: bool,
    pub key_path: String,
    pub source: HostSource,
    pub os: OsKind,
}

impl HostModalState {
    pub fn new_empty() -> Self {
        Self {
            editing_id: None,
            label: String::new(),
            hostname: String::new(),
            port: "22".to_string(),
            username: std::env::var("USER").unwrap_or_else(|_| "root".to_string()),
            auth_mode: 0,
            password: String::new(),
            show_password: false,
            key_path: "~/.ssh/id_ed25519".to_string(),
            source: HostSource::Manual,
            os: OsKind::Unknown,
        }
    }

    pub fn from_host(host: &Host) -> Self {
        let (auth_mode, key_path) = match &host.auth {
            AuthMethod::Agent => (0, "~/.ssh/id_ed25519".to_string()),
            AuthMethod::Password => (1, "~/.ssh/id_ed25519".to_string()),
            AuthMethod::PrivateKey { path } => (2, path.clone()),
        };
        Self {
            editing_id: Some(host.id),
            label: host.label.clone(),
            hostname: host.hostname.clone(),
            port: host.port.to_string(),
            username: host.username.clone(),
            auth_mode,
            password: host.password.clone().unwrap_or_default(),
            show_password: false,
            key_path,
            source: host.source.clone(),
            os: host.os,
        }
    }

    pub fn to_host(&self) -> Host {
        let auth = match self.auth_mode {
            1 => AuthMethod::Password,
            2 => AuthMethod::PrivateKey {
                path: self.key_path.trim().to_string(),
            },
            _ => AuthMethod::Agent,
        };
        Host {
            id: self.editing_id.unwrap_or_else(Uuid::new_v4),
            label: if self.label.trim().is_empty() {
                self.hostname.trim().to_string()
            } else {
                self.label.trim().to_string()
            },
            hostname: self.hostname.trim().to_string(),
            port: self.port.trim().parse().unwrap_or(22),
            username: self.username.trim().to_string(),
            auth,
            password: if self.password.is_empty() {
                None
            } else {
                Some(self.password.clone())
            },
            group_id: None,
            source: self.source.clone(),
            os: self.os,
            updated_at: Utc::now(),
        }
    }
}

pub enum HostModalAction {
    None,
    Save(Host),
    SaveAndConnect(Host),
    Cancel,
}

fn styled_field(
    ui: &mut Ui,
    id: &str,
    text: &mut String,
    hint: &str,
    is_password: bool,
    leading_icon: Option<&str>,
) {
    let h = 34.0;
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(w, h), Sense::hover());

    ui.painter().rect_filled(rect, CornerRadius::same(7), theme::light(110));

    let text_left = if let Some(glyph) = leading_icon {
        icons::paint(
            ui.painter(),
            pos2(rect.left() + 14.0, rect.center().y),
            glyph,
            14.0,
            theme::TEXT_MUTED,
            IconStyle::Regular,
        );
        rect.left() + 28.0
    } else {
        rect.left() + 10.0
    };

    let edit_rect = egui::Rect::from_min_max(
        pos2(text_left, rect.top()),
        pos2(rect.right() - 10.0, rect.bottom()),
    );
    let resp = ui.put(
        edit_rect,
        egui::TextEdit::singleline(text)
            .id_salt(id)
            .password(is_password)
            .frame(false)
            .margin(Margin::ZERO)
            .font(theme::regular(13.0))
            .text_color(theme::TEXT_PRIMARY)
            .vertical_align(Align::Center)
            .desired_width(edit_rect.width())
            .hint_text(
                RichText::new(hint)
                    .color(theme::TEXT_PLACEHOLDER)
                    .font(theme::regular(13.0)),
            ),
    );

    let stroke = if resp.has_focus() {
        Stroke::new(1.2_f32, theme::FOCUS_RING)
    } else {
        Stroke::new(1.0_f32, theme::shade(22))
    };
    ui.painter()
        .rect_stroke(rect, CornerRadius::same(7), stroke, StrokeKind::Inside);
}

fn section_card(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    egui::Frame::NONE
        .fill(theme::light(85))
        .stroke(Stroke::new(1.0_f32, theme::shade(12)))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add_contents(ui);
        });
}

pub fn render_host_panel(ui: &mut Ui, state: &mut HostModalState) -> HostModalAction {
    let mut action = HostModalAction::None;
    let rect = ui.max_rect();
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, theme::light(48));
    painter.vline(rect.left() + 0.5, rect.y_range(), Stroke::new(1.0_f32, theme::SEPARATOR));

    let inner = rect.shrink2(vec2(14.0, 12.0));
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner));

    child.vertical(|ui| {
        // Header: New Host + close arrow
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                let title = if state.editing_id.is_some() {
                    "Edit Host"
                } else {
                    "New Host"
                };
                ui.label(
                    RichText::new(title)
                        .font(theme::semibold(15.0))
                        .color(theme::TEXT_PRIMARY),
                );
                ui.label(
                    RichText::new("Personal vault")
                        .font(theme::regular(12.0))
                        .color(theme::TEXT_MUTED),
                );
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (close_rect, close_resp) =
                    ui.allocate_exact_size(vec2(28.0, 28.0), Sense::click());
                if close_resp.hovered() {
                    ui.painter().rect_filled(close_rect, CornerRadius::same(6), theme::shade(16));
                }
                icons::paint(
                    ui.painter(),
                    close_rect.center(),
                    regular::ARROW_LINE_RIGHT,
                    15.0,
                    theme::ICON_DARK,
                    IconStyle::Regular,
                );
                if close_resp.clicked() {
                    action = HostModalAction::Cancel;
                }
            });
        });

        ui.add_space(12.0);

        let footer_h = 46.0;
        let scroll_h = (ui.available_height() - footer_h).max(80.0);

        egui::ScrollArea::vertical()
            .max_height(scroll_h)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Card 1: Address
                section_card(ui, |ui| {
                    ui.label(
                        RichText::new("Address")
                            .font(theme::semibold(13.0))
                            .color(theme::TEXT_PRIMARY),
                    );
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        let (icon_rect, _) = ui.allocate_exact_size(vec2(36.0, 36.0), Sense::hover());
                        ui.painter().rect_filled(
                            icon_rect,
                            CornerRadius::same(8),
                            Color32::from_rgb(0x0E, 0x4A, 0x72),
                        );
                        icons::paint(
                            ui.painter(),
                            icon_rect.center(),
                            fill::HARD_DRIVES,
                            18.0,
                            Color32::WHITE,
                            IconStyle::Fill,
                        );
                        ui.add_space(6.0);
                        styled_field(
                            ui,
                            "panel_hostname",
                            &mut state.hostname,
                            "IP or Hostname",
                            false,
                            None,
                        );
                    });
                });

                ui.add_space(10.0);

                // Card 2: General (Label only)
                section_card(ui, |ui| {
                    ui.label(
                        RichText::new("General")
                            .font(theme::semibold(13.0))
                            .color(theme::TEXT_PRIMARY),
                    );
                    ui.add_space(8.0);
                    styled_field(ui, "panel_label", &mut state.label, "Label", false, None);
                });

                ui.add_space(10.0);

                // Card 3: SSH port + Credentials
                section_card(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("SSH on")
                                .font(theme::semibold(13.0))
                                .color(theme::TEXT_PRIMARY),
                        );
                        let (port_rect, _) = ui.allocate_exact_size(vec2(64.0, 28.0), Sense::hover());
                        ui.painter().rect_filled(port_rect, CornerRadius::same(6), theme::light(110));
                        let p_resp = ui.put(
                            port_rect.shrink2(vec2(8.0, 0.0)),
                            egui::TextEdit::singleline(&mut state.port)
                                .id_salt("panel_port")
                                .frame(false)
                                .margin(Margin::ZERO)
                                .font(theme::regular(13.0))
                                .text_color(theme::TEXT_PRIMARY)
                                .vertical_align(Align::Center),
                        );
                        let p_stroke = if p_resp.has_focus() {
                            Stroke::new(1.2_f32, theme::FOCUS_RING)
                        } else {
                            Stroke::new(1.0_f32, theme::shade(22))
                        };
                        ui.painter()
                            .rect_stroke(port_rect, CornerRadius::same(6), p_stroke, StrokeKind::Inside);
                        ui.label(
                            RichText::new("port")
                                .font(theme::regular(13.0))
                                .color(theme::TEXT_MUTED),
                        );
                    });

                    ui.add_space(12.0);
                    ui.label(
                        RichText::new("Credentials")
                            .font(theme::semibold(13.0))
                            .color(theme::TEXT_PRIMARY),
                    );
                    ui.add_space(8.0);

                    styled_field(
                        ui,
                        "panel_user",
                        &mut state.username,
                        "Username",
                        false,
                        Some(regular::USER),
                    );
                    ui.add_space(8.0);

                    // Password row with eye toggle
                    ui.horizontal(|ui| {
                        let eye_w = 30.0;
                        let field_w = (ui.available_width() - eye_w - 4.0).max(80.0);
                        ui.allocate_ui(vec2(field_w, 34.0), |ui| {
                            styled_field(
                                ui,
                                "panel_pass",
                                &mut state.password,
                                "Password",
                                !state.show_password,
                                Some(regular::LOCK),
                            );
                        });
                        let (eye_rect, eye_resp) =
                            ui.allocate_exact_size(vec2(eye_w, 34.0), Sense::click());
                        if eye_resp.hovered() {
                            ui.painter().rect_filled(eye_rect, CornerRadius::same(6), theme::shade(12));
                        }
                        let eye_glyph = if state.show_password {
                            regular::EYE_SLASH
                        } else {
                            regular::EYE
                        };
                        icons::paint(
                            ui.painter(),
                            eye_rect.center(),
                            eye_glyph,
                            15.0,
                            theme::TEXT_MUTED,
                            IconStyle::Regular,
                        );
                        if eye_resp.clicked() {
                            state.show_password = !state.show_password;
                        }
                    });

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut state.auth_mode, 0, "Auto / Agent");
                        ui.radio_value(&mut state.auth_mode, 1, "Password");
                        ui.radio_value(&mut state.auth_mode, 2, "Key");
                    });

                    if state.auth_mode == 2 {
                        ui.add_space(8.0);
                        styled_field(
                            ui,
                            "panel_key",
                            &mut state.key_path,
                            "~/.ssh/id_ed25519",
                            false,
                            Some(regular::KEY),
                        );
                    }
                });
            });

        // Footer CTA
        ui.add_space(8.0);
        let can_submit = !state.hostname.trim().is_empty();
        ui.horizontal(|ui| {
            let half = (ui.available_width() - 8.0) / 2.0;
            let save_btn = egui::Button::new(
                RichText::new("Save")
                    .font(theme::semibold(13.0))
                    .color(theme::TEXT_PRIMARY),
            )
            .fill(theme::shade(18))
            .corner_radius(CornerRadius::same(7))
            .min_size(vec2(half, 36.0));
            if ui.add_enabled(can_submit, save_btn).clicked() {
                action = HostModalAction::Save(state.to_host());
            }

            let conn_btn = egui::Button::new(
                RichText::new("Connect")
                    .font(theme::semibold(13.0))
                    .color(Color32::WHITE),
            )
            .fill(if can_submit {
                theme::ACCENT_BLUE
            } else {
                theme::shade(22)
            })
            .corner_radius(CornerRadius::same(7))
            .min_size(vec2(half, 36.0));
            if ui.add_enabled(can_submit, conn_btn).clicked() {
                action = HostModalAction::SaveAndConnect(state.to_host());
            }
        });
    });

    action
}
