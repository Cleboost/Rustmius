//! Termius "Hosts" view: quick-connect bar, toolbar, empty state and host cards.

use crate::i18n::{Language, tr};
use crate::storage::{Host, Vault};
use crate::ui::icons::{self, IconStyle, bold, fill, regular};
use crate::ui::os_icons::paint_os_tile;
use crate::ui::popover_menu::{MenuEntry, MenuIcon, render_popover_menu};
use crate::ui::theme;
use egui::epaint::{Mesh, Shape};
use egui::{
    Align2, Color32, CornerRadius, Margin, Painter, Rect, RichText, Sense, Stroke, StrokeKind, Ui,
    pos2, vec2,
};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostViewMode {
    Grid,
    List,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostSortMode {
    Az,
    Za,
    NewestFirst,
    OldestFirst,
}

pub enum HostGridAction {
    None,
    Connect(Host),
    NewHost,
    NewHostWithAddress(String),
    EditHost(Host),
    DeleteHost(Uuid),
    ImportSshConfig,
    ImportGhost(Host),
    QuickConnect(String),
}

fn vertical_gradient(painter: &Painter, rect: Rect, radius: f32, top: Color32, bottom: Color32) {
    let r = radius.min(rect.width() * 0.5).min(rect.height() * 0.5);
    let h = rect.height().max(1.0);
    let color_at = |y: f32| top.lerp_to_gamma(bottom, ((y - rect.top()) / h).clamp(0.0, 1.0));

    let mut mesh = Mesh::default();
    let center = rect.center();
    mesh.colored_vertex(center, color_at(center.y));

    const SEGMENTS: usize = 8;
    let corners = [
        (
            pos2(rect.right() - r, rect.bottom() - r),
            0.0_f32,
            std::f32::consts::FRAC_PI_2,
        ),
        (
            pos2(rect.left() + r, rect.bottom() - r),
            std::f32::consts::FRAC_PI_2,
            std::f32::consts::PI,
        ),
        (
            pos2(rect.left() + r, rect.top() + r),
            std::f32::consts::PI,
            3.0 * std::f32::consts::FRAC_PI_2,
        ),
        (
            pos2(rect.right() - r, rect.top() + r),
            3.0 * std::f32::consts::FRAC_PI_2,
            std::f32::consts::TAU,
        ),
    ];

    for (cc, a0, a1) in corners {
        for i in 0..=SEGMENTS {
            let a = a0 + (a1 - a0) * (i as f32 / SEGMENTS as f32);
            let p = pos2(cc.x + r * a.cos(), cc.y + r * a.sin());
            mesh.colored_vertex(p, color_at(p.y));
        }
    }

    let n = (mesh.vertices.len() - 1) as u32;
    for i in 1..=n {
        let next = if i == n { 1 } else { i + 1 };
        mesh.add_triangle(0, i, next);
    }
    painter.add(Shape::mesh(mesh));
}

fn single_line_input(
    ui: &mut Ui,
    rect: Rect,
    text: &mut String,
    hint: &str,
    id: &str,
) -> egui::Response {
    let font = theme::regular(13.0);
    let row_h = ui.fonts(|f| f.row_height(&font));
    let v_pad = ((rect.height() - row_h) * 0.5).max(0.0) as i8;
    ui.put(
        rect,
        egui::TextEdit::singleline(text)
            .id_salt(id)
            .frame(false)
            .margin(Margin::symmetric(0, v_pad))
            .font(font.clone())
            .text_color(theme::TEXT_PRIMARY)
            .desired_width(rect.width())
            .hint_text(
                RichText::new(hint)
                    .color(theme::TEXT_PLACEHOLDER)
                    .font(font),
            ),
    )
}

/// Header strip (quick connect + toolbar). Returns `(remaining_content_rect, new_host_btn_rect, view_btn_rect, sort_btn_rect)`.
#[allow(clippy::too_many_arguments)]
fn render_header(
    ui: &mut Ui,
    area: Rect,
    quick_connect: &mut String,
    view_mode: HostViewMode,
    view_menu_open: &mut bool,
    sort_menu_open: &mut bool,
    new_host_menu_open: &mut bool,
    lang: Language,
    action: &mut HostGridAction,
) -> (Rect, Rect, Rect, Rect) {
    let s = tr(lang);
    let painter = ui.painter_at(area);
    let header = Rect::from_min_size(area.min, vec2(area.width(), theme::HEADER_H));
    painter.rect_filled(header, 0.0, theme::light(46));
    painter.hline(
        header.x_range(),
        header.bottom() - 0.5,
        Stroke::new(1.0_f32, theme::SEPARATOR),
    );

    let (l, r, t) = (area.left(), area.right(), area.top());

    // --- Quick connect bar
    let qc = Rect::from_min_max(pos2(l + 9.0, t + 8.0), pos2(r - 8.0, t + 46.0));
    painter.rect_filled(qc, CornerRadius::same(6), theme::shade(14));
    let connect = Rect::from_min_max(pos2(r - 86.0, t + 14.0), pos2(r - 11.0, t + 40.0));
    let enabled = !quick_connect.trim().is_empty();
    let mut cresp = ui.interact(
        connect,
        ui.id().with("qc_connect"),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    if enabled {
        cresp = cresp.on_hover_cursor(egui::CursorIcon::PointingHand);
    }
    let cfill = if enabled && cresp.hovered() {
        theme::ACCENT_BLUE_BOTTOM
    } else if enabled {
        theme::ACCENT_BLUE
    } else {
        theme::shade(22)
    };
    painter.rect_filled(connect, CornerRadius::same(5), cfill);
    painter.text(
        connect.center(),
        Align2::CENTER_CENTER,
        s.connect,
        theme::medium(12.5),
        if enabled {
            Color32::WHITE
        } else {
            theme::TEXT_DISABLED
        },
    );
    let input_rect = Rect::from_min_max(
        pos2(qc.left() + 13.0, qc.top()),
        pos2(connect.left() - 8.0, qc.bottom()),
    );
    let qresp = single_line_input(
        ui,
        input_rect,
        quick_connect,
        s.quick_connect_hint,
        "quick_connect",
    );
    let submitted = qresp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    if enabled && (cresp.clicked() || submitted) {
        *action = HostGridAction::QuickConnect(quick_connect.trim().to_string());
    }

    // --- Toolbar
    let cy = t + 79.5;
    let btn_h = 30.0;

    let new_host = Rect::from_min_max(
        pos2(l + 13.0, cy - btn_h / 2.0),
        pos2(l + 113.0, cy + btn_h / 2.0),
    );
    let nresp = ui
        .interact(new_host, ui.id().with("tb_new_host"), Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    painter.rect_filled(
        new_host,
        CornerRadius {
            nw: 6,
            ne: 2,
            sw: 6,
            se: 2,
        },
        theme::shade(if nresp.hovered() { 30 } else { 20 }),
    );
    let plus_c = pos2(l + 29.5, cy);
    let plus_stroke = Stroke::new(1.7_f32, theme::TEXT_PRIMARY);
    painter.line_segment(
        [
            pos2(plus_c.x - 5.5, plus_c.y),
            pos2(plus_c.x + 5.5, plus_c.y),
        ],
        plus_stroke,
    );
    painter.line_segment(
        [
            pos2(plus_c.x, plus_c.y - 5.5),
            pos2(plus_c.x, plus_c.y + 5.5),
        ],
        plus_stroke,
    );
    painter.text(
        pos2(l + 43.0, cy),
        Align2::LEFT_CENTER,
        s.new_host,
        theme::medium(13.0),
        theme::TEXT_PRIMARY,
    );
    if nresp.clicked() {
        *action = HostGridAction::NewHost;
    }

    let chevron = Rect::from_min_max(
        pos2(l + 115.0, cy - btn_h / 2.0),
        pos2(l + 145.0, cy + btn_h / 2.0),
    );
    let chresp = ui
        .interact(chevron, ui.id().with("tb_new_menu"), Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    painter.rect_filled(
        chevron,
        CornerRadius {
            nw: 2,
            ne: 6,
            sw: 2,
            se: 6,
        },
        theme::shade(if chresp.hovered() || *new_host_menu_open {
            30
        } else {
            20
        }),
    );
    icons::paint(
        &painter,
        chevron.center() + vec2(0.0, 0.5),
        bold::CARET_DOWN,
        12.0,
        theme::TEXT_PRIMARY,
        IconStyle::Bold,
    );
    if chresp.clicked() {
        *new_host_menu_open = !*new_host_menu_open;
        if *new_host_menu_open {
            *view_menu_open = false;
            *sort_menu_open = false;
        }
    }

    // Right-aligned view toggles
    let view_btn_rect = Rect::from_center_size(pos2(r - 103.0, cy), vec2(30.0, 30.0));
    let vresp = ui
        .interact(view_btn_rect, ui.id().with("tb_grid"), Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    if vresp.hovered() || *view_menu_open {
        painter.rect_filled(
            view_btn_rect,
            CornerRadius::same(6),
            theme::shade(if *view_menu_open { 18 } else { 12 }),
        );
    }
    let (vglyph, vstyle) = match view_mode {
        HostViewMode::Grid => (fill::SQUARES_FOUR, IconStyle::Fill),
        HostViewMode::List => (regular::LIST_BULLETS, IconStyle::Regular),
    };
    icons::paint(
        &painter,
        view_btn_rect.center(),
        vglyph,
        15.0,
        theme::ICON_DARK,
        vstyle,
    );
    if vresp.clicked() {
        *view_menu_open = !*view_menu_open;
        if *view_menu_open {
            *sort_menu_open = false;
        }
    }

    let tags_rect = Rect::from_center_size(pos2(r - 65.0, cy), vec2(30.0, 30.0));
    let tresp = ui
        .interact(tags_rect, ui.id().with("tb_tags"), Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    if tresp.hovered() {
        painter.rect_filled(tags_rect, CornerRadius::same(6), theme::shade(12));
    }
    icons::paint(
        &painter,
        tags_rect.center(),
        fill::TAG,
        15.0,
        theme::ICON_DARK,
        IconStyle::Fill,
    );

    let sort_btn_rect = Rect::from_center_size(pos2(r - 27.0, cy), vec2(30.0, 30.0));
    let sresp = ui
        .interact(sort_btn_rect, ui.id().with("tb_sort"), Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    if sresp.hovered() || *sort_menu_open {
        painter.rect_filled(
            sort_btn_rect,
            CornerRadius::same(6),
            theme::shade(if *sort_menu_open { 18 } else { 12 }),
        );
    }
    icons::paint(
        &painter,
        sort_btn_rect.center(),
        fill::CALENDAR,
        15.0,
        theme::ICON_DARK,
        IconStyle::Fill,
    );
    if sresp.clicked() {
        *sort_menu_open = !*sort_menu_open;
        if *sort_menu_open {
            *view_menu_open = false;
        }
    }

    (
        Rect::from_min_max(pos2(l, header.bottom()), area.max),
        chevron,
        view_btn_rect,
        sort_btn_rect,
    )
}

fn render_empty_state(
    ui: &mut Ui,
    area: Rect,
    input: &mut String,
    lang: Language,
    action: &mut HostGridAction,
) {
    let s = tr(lang);
    let painter = ui.painter_at(area);
    let cx = area.center().x;
    let top = area.center().y - 125.0;

    // Icon tile
    let tile = Rect::from_center_size(pos2(cx, top + 28.0), vec2(56.0, 56.0));
    painter.rect_filled(tile, CornerRadius::same(10), theme::shade(14));
    icons::paint(
        &painter,
        tile.center(),
        fill::HARD_DRIVES,
        24.0,
        theme::ICON_DARK,
        IconStyle::Fill,
    );

    painter.text(
        pos2(cx, top + 100.0),
        Align2::CENTER_CENTER,
        s.empty_title,
        theme::semibold(16.0),
        theme::TEXT_PRIMARY,
    );
    painter.text(
        pos2(cx, top + 133.0),
        Align2::CENTER_CENTER,
        s.empty_subtitle,
        theme::regular(13.0),
        theme::TEXT_MUTED,
    );

    // Address input
    let field = Rect::from_min_size(pos2(cx - 196.0, top + 166.0), vec2(392.0, 34.0));
    painter.rect_filled(field, CornerRadius::same(6), theme::light(40));
    let resp = single_line_input(
        ui,
        field.shrink2(vec2(11.0, 0.0)),
        input,
        s.empty_hint,
        "empty_host_input",
    );
    let init_id = resp.id.with("initial_focus");
    let did_focus = ui.data_mut(|d| {
        let already = d.get_temp::<bool>(init_id).unwrap_or(false);
        if !already {
            d.insert_temp(init_id, true);
        }
        already
    });
    if !did_focus {
        resp.request_focus();
    }
    let ring = if resp.has_focus() {
        Stroke::new(1.0_f32, theme::FOCUS_RING)
    } else {
        Stroke::new(1.0_f32, theme::shade(36))
    };
    painter.rect_stroke(field, CornerRadius::same(6), ring, StrokeKind::Inside);

    // Continue
    let button = Rect::from_min_size(pos2(cx - 196.0, top + 222.0), vec2(392.0, 36.0));
    let bresp = ui
        .interact(button, ui.id().with("empty_continue"), Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    let (t, b) = if bresp.hovered() {
        (
            theme::ACCENT_BLUE_TOP.gamma_multiply(1.08),
            theme::ACCENT_BLUE,
        )
    } else {
        (theme::ACCENT_BLUE_TOP, theme::ACCENT_BLUE_BOTTOM)
    };
    vertical_gradient(&painter, button, 6.0, t, b);
    painter.text(
        button.center(),
        Align2::CENTER_CENTER,
        s.continue_btn,
        theme::semibold(13.0),
        Color32::WHITE,
    );

    let submitted =
        bresp.clicked() || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
    if submitted && !input.trim().is_empty() {
        *action = HostGridAction::NewHostWithAddress(input.trim().to_string());
        input.clear();
    }
}

#[allow(clippy::too_many_arguments)]
fn render_cards(
    ui: &mut Ui,
    area: Rect,
    vault: &Vault,
    ghost_hosts: &[Host],
    view_mode: HostViewMode,
    sort_mode: HostSortMode,
    lang: Language,
    action: &mut HostGridAction,
) {
    let s = tr(lang);
    let mut hosts: Vec<&Host> = vault.hosts.iter().collect();
    match sort_mode {
        HostSortMode::Az => hosts.sort_by_key(|h| h.label.to_ascii_lowercase()),
        HostSortMode::Za => hosts.sort_by_key(|h| std::cmp::Reverse(h.label.to_ascii_lowercase())),
        HostSortMode::NewestFirst => hosts.sort_by_key(|h| std::cmp::Reverse(h.updated_at)),
        HostSortMode::OldestFirst => hosts.sort_by_key(|h| h.updated_at),
    }

    let inner = area.shrink2(vec2(13.0, 0.0));
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner));
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(&mut child, |ui| {
            ui.add_space(18.0);
            ui.label(
                RichText::new(s.hosts_heading)
                    .font(theme::semibold(14.0))
                    .color(theme::TEXT_PRIMARY),
            );
            ui.add_space(6.0);

            let gap = if view_mode == HostViewMode::List {
                8.0
            } else {
                12.0
            };
            let card_h = if view_mode == HostViewMode::List {
                56.0
            } else {
                64.0
            };
            let width = ui.available_width();
            let cols = match view_mode {
                HostViewMode::List => 1,
                HostViewMode::Grid => (((width + gap) / (260.0 + gap)).floor() as usize).max(1),
            };
            let card_w = (width - gap * (cols as f32 - 1.0)) / cols as f32;
            let rows = hosts.len().div_ceil(cols);
            let (grid, _) =
                ui.allocate_exact_size(vec2(width, rows as f32 * (card_h + gap)), Sense::hover());
            let painter = ui.painter();

            for (idx, host) in hosts.iter().enumerate() {
                let (row, col) = (idx / cols, idx % cols);
                let rect = Rect::from_min_size(
                    pos2(
                        grid.left() + col as f32 * (card_w + gap),
                        grid.top() + row as f32 * (card_h + gap),
                    ),
                    vec2(card_w, card_h),
                );
                let resp = ui
                    .interact(rect, ui.id().with(("host_card", host.id)), Sense::click())
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                let edit_rect = Rect::from_center_size(
                    pos2(rect.right() - 22.0, rect.center().y),
                    vec2(28.0, 28.0),
                );
                let edit_resp = ui
                    .interact(
                        edit_rect,
                        ui.id().with(("host_edit", host.id)),
                        Sense::click(),
                    )
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                let hovered = resp.hovered() || edit_resp.hovered();

                let card_bg = if hovered {
                    Color32::from_rgba_unmultiplied(0xCA, 0xD1, 0xD5, 165)
                } else {
                    Color32::from_rgba_unmultiplied(0xF6, 0xF8, 0xF9, 200)
                };
                let shadow_alpha = if hovered { 26 } else { 16 };
                painter.rect_filled(
                    rect.translate(vec2(0.0, 3.0)),
                    CornerRadius::same(8),
                    Color32::from_black_alpha(shadow_alpha),
                );
                painter.rect_filled(rect, CornerRadius::same(8), card_bg);
                painter.rect_stroke(
                    rect,
                    CornerRadius::same(8),
                    Stroke::new(1.0_f32, theme::shade(12)),
                    StrokeKind::Inside,
                );

                let tile = Rect::from_min_size(
                    pos2(rect.left() + 12.0, rect.center().y - 20.0),
                    vec2(40.0, 40.0),
                );
                paint_os_tile(painter, tile, host.os, &host.source);

                let tx = tile.right() + 12.0;
                let right_pad = if hovered { 42.0 } else { 12.0 };
                let max_w = rect.right() - tx - right_pad;
                let label = painter.layout(
                    host.label.clone(),
                    theme::semibold(14.0),
                    theme::TEXT_PRIMARY,
                    max_w,
                );
                painter.galley(pos2(tx, rect.center().y - 18.0), label, theme::TEXT_PRIMARY);
                painter.text(
                    pos2(tx, rect.center().y + 10.0),
                    Align2::LEFT_CENTER,
                    format!("ssh, {}", host.username),
                    theme::regular(12.0),
                    theme::TEXT_MUTED,
                );

                if hovered {
                    if edit_resp.hovered() {
                        painter.rect_filled(edit_rect, CornerRadius::same(6), theme::shade(16));
                    }
                    icons::paint(
                        painter,
                        edit_rect.center(),
                        fill::PENCIL_SIMPLE,
                        15.0,
                        theme::ICON_DARK,
                        IconStyle::Fill,
                    );
                }

                if edit_resp.clicked() {
                    *action = HostGridAction::EditHost((*host).clone());
                } else if resp.double_clicked() {
                    *action = HostGridAction::Connect((*host).clone());
                }
                resp.context_menu(|ui| {
                    if ui.button(s.ctx_connect).clicked() {
                        *action = HostGridAction::Connect((*host).clone());
                        ui.close_menu();
                    }
                    if ui.button(s.ctx_edit).clicked() {
                        *action = HostGridAction::EditHost((*host).clone());
                        ui.close_menu();
                    }
                    if ui
                        .button(RichText::new(s.ctx_delete).color(theme::DANGER))
                        .clicked()
                    {
                        *action = HostGridAction::DeleteHost(host.id);
                        ui.close_menu();
                    }
                });
            }

            if !ghost_hosts.is_empty() {
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(s.detected_ssh_heading)
                            .font(theme::semibold(13.0))
                            .color(theme::TEXT_MUTED),
                    );
                    if ui.small_button(s.import_all).clicked() {
                        *action = HostGridAction::ImportSshConfig;
                    }
                });
                ui.add_space(6.0);

                let ghost_rows = ghost_hosts.len().div_ceil(cols);
                let (ghost_grid, _) = ui.allocate_exact_size(
                    vec2(width, ghost_rows as f32 * (card_h + gap)),
                    Sense::hover(),
                );
                let painter = ui.painter();

                for (idx, host) in ghost_hosts.iter().enumerate() {
                    let (row, col) = (idx / cols, idx % cols);
                    let rect = Rect::from_min_size(
                        pos2(
                            ghost_grid.left() + col as f32 * (card_w + gap),
                            ghost_grid.top() + row as f32 * (card_h + gap),
                        ),
                        vec2(card_w, card_h),
                    );
                    let resp = ui.interact(
                        rect,
                        ui.id().with(("ghost_card", &host.label, idx)),
                        Sense::hover(),
                    );
                    let bg = if resp.hovered() {
                        theme::light(80)
                    } else {
                        theme::light(42)
                    };
                    painter.rect_filled(rect, CornerRadius::same(8), bg);
                    painter.rect_stroke(
                        rect,
                        CornerRadius::same(8),
                        Stroke::new(1.0_f32, theme::shade(24)),
                        StrokeKind::Inside,
                    );

                    let tile = Rect::from_min_size(
                        pos2(rect.left() + 12.0, rect.center().y - 20.0),
                        vec2(40.0, 40.0),
                    );
                    painter.rect_filled(tile, CornerRadius::same(8), theme::shade(28));
                    icons::paint(
                        painter,
                        tile.center(),
                        fill::HARD_DRIVES,
                        18.0,
                        theme::TEXT_MUTED,
                        IconStyle::Fill,
                    );

                    let import_rect = Rect::from_center_size(
                        pos2(rect.right() - 40.0, rect.center().y),
                        vec2(60.0, 26.0),
                    );
                    let imp_resp = ui
                        .interact(
                            import_rect,
                            ui.id().with(("ghost_import", &host.label, idx)),
                            Sense::click(),
                        )
                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                    painter.rect_filled(
                        import_rect,
                        CornerRadius::same(6),
                        if imp_resp.hovered() {
                            theme::ACCENT_BLUE
                        } else {
                            theme::shade(22)
                        },
                    );
                    painter.text(
                        import_rect.center(),
                        Align2::CENTER_CENTER,
                        s.import_btn,
                        theme::medium(12.0),
                        if imp_resp.hovered() {
                            Color32::WHITE
                        } else {
                            theme::TEXT_PRIMARY
                        },
                    );
                    if imp_resp.clicked() {
                        *action = HostGridAction::ImportGhost(host.clone());
                    }

                    let tx = tile.right() + 12.0;
                    let max_w = (import_rect.left() - tx - 8.0).max(40.0);
                    let label = painter.layout(
                        host.label.clone(),
                        theme::semibold(14.0),
                        theme::TEXT_MUTED,
                        max_w,
                    );
                    painter.galley(pos2(tx, rect.center().y - 18.0), label, theme::TEXT_MUTED);
                    painter.text(
                        pos2(tx, rect.center().y + 10.0),
                        Align2::LEFT_CENTER,
                        format!("ssh, {}@{}", host.username, host.hostname),
                        theme::regular(11.5),
                        theme::TEXT_PLACEHOLDER,
                    );
                }
            }
        });
}

#[allow(clippy::too_many_arguments)]
pub fn render_host_grid(
    ui: &mut Ui,
    vault: &Vault,
    ghost_hosts: &[Host],
    quick_connect: &mut String,
    empty_host_input: &mut String,
    view_mode: &mut HostViewMode,
    view_menu_open: &mut bool,
    sort_mode: &mut HostSortMode,
    sort_menu_open: &mut bool,
    new_host_menu_open: &mut bool,
    lang: Language,
) -> HostGridAction {
    let s = tr(lang);
    let mut action = HostGridAction::None;
    let area = ui.max_rect();
    let (content, new_host_btn_rect, view_btn_rect, sort_btn_rect) = render_header(
        ui,
        area,
        quick_connect,
        *view_mode,
        view_menu_open,
        sort_menu_open,
        new_host_menu_open,
        lang,
        &mut action,
    );
    if vault.hosts.is_empty() && ghost_hosts.is_empty() {
        render_empty_state(ui, content, empty_host_input, lang, &mut action);
    } else {
        render_cards(
            ui,
            content,
            vault,
            ghost_hosts,
            *view_mode,
            *sort_mode,
            lang,
            &mut action,
        );
    }

    if let Some(id) = render_popover_menu(
        ui.ctx(),
        "new_host_popover",
        new_host_btn_rect,
        228.0,
        new_host_menu_open,
        &[
            MenuEntry::Item {
                id: 0,
                icon: MenuIcon::Phosphor(regular::PLUS, IconStyle::Regular),
                label: s.new_host,
                selected: false,
            },
            MenuEntry::Item {
                id: 1,
                icon: MenuIcon::Phosphor(regular::TERMINAL_WINDOW, IconStyle::Regular),
                label: s.import_ssh_config,
                selected: false,
            },
        ],
    ) {
        action = if id == 0 {
            HostGridAction::NewHost
        } else {
            HostGridAction::ImportSshConfig
        };
    }

    if let Some(id) = render_popover_menu(
        ui.ctx(),
        "view_mode_popover",
        view_btn_rect,
        152.0,
        view_menu_open,
        &[
            MenuEntry::Item {
                id: 0,
                icon: MenuIcon::Phosphor(fill::SQUARES_FOUR, IconStyle::Fill),
                label: s.view_grid,
                selected: *view_mode == HostViewMode::Grid,
            },
            MenuEntry::Item {
                id: 1,
                icon: MenuIcon::Phosphor(regular::LIST_BULLETS, IconStyle::Regular),
                label: s.view_list,
                selected: *view_mode == HostViewMode::List,
            },
        ],
    ) {
        *view_mode = if id == 0 {
            HostViewMode::Grid
        } else {
            HostViewMode::List
        };
    }

    if let Some(id) = render_popover_menu(
        ui.ctx(),
        "sort_mode_popover",
        sort_btn_rect,
        204.0,
        sort_menu_open,
        &[
            MenuEntry::Item {
                id: 0,
                icon: MenuIcon::Badge("Az"),
                label: s.sort_az,
                selected: *sort_mode == HostSortMode::Az,
            },
            MenuEntry::Item {
                id: 1,
                icon: MenuIcon::Badge("Za"),
                label: s.sort_za,
                selected: *sort_mode == HostSortMode::Za,
            },
            MenuEntry::Separator,
            MenuEntry::Item {
                id: 2,
                icon: MenuIcon::Phosphor(regular::CALENDAR, IconStyle::Regular),
                label: s.sort_newest,
                selected: *sort_mode == HostSortMode::NewestFirst,
            },
            MenuEntry::Item {
                id: 3,
                icon: MenuIcon::Phosphor(regular::CALENDAR, IconStyle::Regular),
                label: s.sort_oldest,
                selected: *sort_mode == HostSortMode::OldestFirst,
            },
        ],
    ) {
        *sort_mode = match id {
            0 => HostSortMode::Az,
            1 => HostSortMode::Za,
            2 => HostSortMode::NewestFirst,
            _ => HostSortMode::OldestFirst,
        };
    }

    action
}
